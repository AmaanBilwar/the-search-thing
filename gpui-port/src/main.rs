use gpui::{
    actions, div, fill, point, prelude::*, px, relative, rgb, rgba, size, App, Application, Bounds,
    ClipboardItem, Context, CursorStyle, ElementId, ElementInputHandler, Entity,
    EntityInputHandler, FocusHandle, Focusable, GlobalElementId, KeyBinding, KeystrokeEvent,
    LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, PaintQuad, Pixels, Point,
    ShapedLine, SharedString, Style, TextRun, UTF16Selection, UnderlineStyle, Window, WindowBounds,
    WindowControlArea, WindowOptions,
};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::ops::Range;
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use unicode_segmentation::UnicodeSegmentation;

actions!(
    text_input,
    [
        Backspace,
        Delete,
        Left,
        Right,
        SelectLeft,
        SelectRight,
        SelectAll,
        Home,
        End,
        ShowCharacterPalette,
        Paste,
        Cut,
        Copy,
    ]
);

actions!(search_window, [RunSearch, Quit]);

struct TextInput {
    focus_handle: FocusHandle,
    content: SharedString,
    placeholder: SharedString,
    selected_range: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
    last_layout: Option<ShapedLine>,
    last_bounds: Option<Bounds<Pixels>>,
    is_selecting: bool,
}

impl TextInput {
    fn new(cx: &mut Context<Self>, placeholder: &str) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            content: "".into(),
            placeholder: placeholder.to_string().into(),
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            last_layout: None,
            last_bounds: None,
            is_selecting: false,
        }
    }

    fn text(&self) -> String {
        self.content.to_string()
    }

    fn set_text(&mut self, text: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.content = text.into();
        let len = self.content.len();
        self.selected_range = len..len;
        self.selection_reversed = false;
        self.marked_range = None;
        cx.notify();
    }

    fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.previous_boundary(self.cursor_offset()), cx);
        } else {
            self.move_to(self.selected_range.start, cx)
        }
    }

    fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.move_to(self.next_boundary(self.selected_range.end), cx);
        } else {
            self.move_to(self.selected_range.end, cx)
        }
    }

    fn select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.previous_boundary(self.cursor_offset()), cx);
    }

    fn select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.next_boundary(self.cursor_offset()), cx);
    }

    fn select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, cx);
        self.select_to(self.content.len(), cx)
    }

    fn home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(0, cx);
    }

    fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.content.len(), cx);
    }

    fn backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.select_to(self.previous_boundary(self.cursor_offset()), cx)
        }
        self.replace_text_in_range(None, "", window, cx)
    }

    fn delete(&mut self, _: &Delete, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_range.is_empty() {
            self.select_to(self.next_boundary(self.cursor_offset()), cx)
        }
        self.replace_text_in_range(None, "", window, cx)
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.is_selecting = true;
        window.focus(&self.focus_handle(cx));

        if event.modifiers.shift {
            self.select_to(self.index_for_mouse_position(event.position), cx);
        } else {
            self.move_to(self.index_for_mouse_position(event.position), cx)
        }
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, _window: &mut Window, _: &mut Context<Self>) {
        self.is_selecting = false;
    }

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.is_selecting {
            self.select_to(self.index_for_mouse_position(event.position), cx);
        }
    }

    fn show_character_palette(
        &mut self,
        _: &ShowCharacterPalette,
        window: &mut Window,
        _: &mut Context<Self>,
    ) {
        window.show_character_palette();
    }

    fn paste(&mut self, _: &Paste, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            self.replace_text_in_range(None, &text.replace('\n', " "), window, cx);
        }
    }

    fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
        }
    }

    fn cut(&mut self, _: &Cut, window: &mut Window, cx: &mut Context<Self>) {
        if !self.selected_range.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected_range.clone()].to_string(),
            ));
            self.replace_text_in_range(None, "", window, cx)
        }
    }

    fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.selected_range = offset..offset;
        cx.notify()
    }

    fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn index_for_mouse_position(&self, position: Point<Pixels>) -> usize {
        if self.content.is_empty() {
            return 0;
        }

        let (Some(bounds), Some(line)) = (self.last_bounds.as_ref(), self.last_layout.as_ref())
        else {
            return 0;
        };
        if position.y < bounds.top() {
            return 0;
        }
        if position.y > bounds.bottom() {
            return self.content.len();
        }
        line.closest_index_for_x(position.x - bounds.left())
    }

    fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        if self.selection_reversed {
            self.selected_range.start = offset
        } else {
            self.selected_range.end = offset
        };
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
        cx.notify()
    }

    fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;

        for ch in self.content.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }

        utf8_offset
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;

        for ch in self.content.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }

        utf16_offset
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range_utf16.start)..self.offset_from_utf16(range_utf16.end)
    }

    fn previous_boundary(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .rev()
            .find_map(|(idx, _)| (idx < offset).then_some(idx))
            .unwrap_or(0)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        self.content
            .grapheme_indices(true)
            .find_map(|(idx, _)| (idx > offset).then_some(idx))
            .unwrap_or(self.content.len())
    }
}

impl EntityInputHandler for TextInput {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        actual_range.replace(self.range_to_utf16(&range));
        Some(self.content[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: self.selection_reversed,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        self.marked_range = None;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        self.content =
            (self.content[0..range.start].to_owned() + new_text + &self.content[range.end..])
                .into();
        self.selected_range = range.start + new_text.len()..range.start + new_text.len();
        self.marked_range.take();
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or(self.selected_range.clone());

        self.content =
            (self.content[0..range.start].to_owned() + new_text + &self.content[range.end..])
                .into();
        if !new_text.is_empty() {
            self.marked_range = Some(range.start..range.start + new_text.len());
        } else {
            self.marked_range = None;
        }
        self.selected_range = new_selected_range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .map(|new_range| new_range.start + range.start..new_range.end + range.end)
            .unwrap_or_else(|| range.start + new_text.len()..range.start + new_text.len());

        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let last_layout = self.last_layout.as_ref()?;
        let range = self.range_from_utf16(&range_utf16);
        Some(Bounds::from_corners(
            point(
                bounds.left() + last_layout.x_for_index(range.start),
                bounds.top(),
            ),
            point(
                bounds.left() + last_layout.x_for_index(range.end),
                bounds.bottom(),
            ),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: gpui::Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let line_point = self.last_bounds?.localize(&point)?;
        let last_layout = self.last_layout.as_ref()?;

        let utf8_index = last_layout.index_for_x(point.x - line_point.x)?;
        Some(self.offset_to_utf16(utf8_index))
    }
}

struct TextElement {
    input: Entity<TextInput>,
}

struct PrepaintState {
    line: Option<ShapedLine>,
    cursor: Option<PaintQuad>,
    selection: Option<PaintQuad>,
}

impl IntoElement for TextElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextElement {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let input = self.input.read(cx);
        let content = input.content.clone();
        let selected_range = input.selected_range.clone();
        let cursor = input.cursor_offset();
        let style = window.text_style();

        let (display_text, text_color) = if content.is_empty() {
            (input.placeholder.clone(), style.color)
        } else {
            (content, style.color)
        };

        let run = TextRun {
            len: display_text.len(),
            font: style.font(),
            color: text_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };

        let runs = if let Some(marked_range) = input.marked_range.as_ref() {
            vec![
                TextRun {
                    len: marked_range.start,
                    ..run.clone()
                },
                TextRun {
                    len: marked_range.end - marked_range.start,
                    underline: Some(UnderlineStyle {
                        color: Some(run.color),
                        thickness: px(1.0),
                        wavy: false,
                    }),
                    ..run.clone()
                },
                TextRun {
                    len: display_text.len() - marked_range.end,
                    ..run
                },
            ]
            .into_iter()
            .filter(|run| run.len > 0)
            .collect()
        } else {
            vec![run]
        };

        let font_size = style.font_size.to_pixels(window.rem_size());
        let line = window
            .text_system()
            .shape_line(display_text, font_size, &runs, None);

        let cursor_pos = line.x_for_index(cursor);
        let (selection, cursor) = if selected_range.is_empty() {
            (
                None,
                Some(fill(
                    Bounds::new(
                        point(bounds.left() + cursor_pos, bounds.top()),
                        size(px(2.), bounds.bottom() - bounds.top()),
                    ),
                    rgb(0x60a5fa),
                )),
            )
        } else {
            (
                Some(fill(
                    Bounds::from_corners(
                        point(
                            bounds.left() + line.x_for_index(selected_range.start),
                            bounds.top(),
                        ),
                        point(
                            bounds.left() + line.x_for_index(selected_range.end),
                            bounds.bottom(),
                        ),
                    ),
                    rgba(0x3348a0ff),
                )),
                None,
            )
        };

        PrepaintState {
            line: Some(line),
            cursor,
            selection,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.input.clone()),
            cx,
        );

        if let Some(selection) = prepaint.selection.take() {
            window.paint_quad(selection)
        }

        let line = prepaint.line.take().unwrap();
        line.paint(bounds.origin, window.line_height(), window, cx)
            .unwrap();

        if focus_handle.is_focused(window) {
            if let Some(cursor) = prepaint.cursor.take() {
                window.paint_quad(cursor);
            }
        }

        self.input.update(cx, |input, _| {
            input.last_layout = Some(line);
            input.last_bounds = Some(bounds);
        });
    }
}

impl Focusable for TextInput {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TextInput {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .key_context("TextInput")
            .track_focus(&self.focus_handle(cx))
            .cursor(CursorStyle::IBeam)
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::show_character_palette))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::copy))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .line_height(px(24.0))
            .text_size(px(14.0))
            .child(
                div()
                    .h(px(24.0 + 8.0))
                    .w_full()
                    .p(px(4.0))
                    .child(TextElement { input: cx.entity() }),
            )
    }
}

#[derive(Clone)]
struct SearchResult {
    label: SharedString,
    path: SharedString,
    content: SharedString,
}

#[derive(Debug, Deserialize)]
struct JsonRpcErrorPayload {
    code: i32,
    message: String,
    data: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponseEnvelope {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Value,
    result: Option<Value>,
    error: Option<JsonRpcErrorPayload>,
}

#[derive(Debug, Deserialize)]
struct SearchQueryResponse {
    results: Vec<SearchQueryItem>,
}

#[derive(Debug, Deserialize)]
struct SearchQueryItem {
    label: String,
    path: String,
    content: Option<String>,
}

struct SidecarProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

struct SidecarLaunchSpec {
    command: String,
    args: Vec<String>,
}

struct SidecarClient {
    process: Option<SidecarProcess>,
    next_id: u64,
    repo_root: PathBuf,
}

impl SidecarClient {
    fn new() -> Self {
        Self {
            process: None,
            next_id: 1,
            repo_root: Self::resolve_repo_root(),
        }
    }

    fn resolve_repo_root() -> PathBuf {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        if cwd.join("config/file_types.json").exists() {
            return cwd;
        }
        if let Some(parent) = cwd.parent() {
            if parent.join("config/file_types.json").exists() {
                return parent.to_path_buf();
            }
        }
        cwd
    }

    fn sidecar_name() -> &'static str {
        if cfg!(target_os = "windows") {
            "the-search-thing-sidecar.exe"
        } else {
            "the-search-thing-sidecar"
        }
    }

    fn launch_spec(&self) -> SidecarLaunchSpec {
        let debug = self
            .repo_root
            .join("target/debug")
            .join(Self::sidecar_name());
        if debug.exists() {
            return SidecarLaunchSpec {
                command: debug.to_string_lossy().to_string(),
                args: vec![],
            };
        }

        let release = self
            .repo_root
            .join("target/release")
            .join(Self::sidecar_name());
        if release.exists() {
            return SidecarLaunchSpec {
                command: release.to_string_lossy().to_string(),
                args: vec![],
            };
        }

        SidecarLaunchSpec {
            command: "cargo".to_string(),
            args: vec![
                "run".to_string(),
                "--quiet".to_string(),
                "--manifest-path".to_string(),
                self.repo_root
                    .join("Cargo.toml")
                    .to_string_lossy()
                    .to_string(),
                "--bin".to_string(),
                "the-search-thing-sidecar".to_string(),
            ],
        }
    }

    fn ensure_started(&mut self) -> Result<(), String> {
        if self.process.is_some() {
            return Ok(());
        }

        let launch = self.launch_spec();
        let mut child = Command::new(&launch.command)
            .args(&launch.args)
            .current_dir(&self.repo_root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| format!("failed to spawn sidecar '{}': {}", launch.command, e))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "failed to capture sidecar stdin".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "failed to capture sidecar stdout".to_string())?;

        self.process = Some(SidecarProcess {
            child,
            stdin,
            stdout: BufReader::new(stdout),
        });
        Ok(())
    }

    fn call<T: DeserializeOwned>(
        &mut self,
        method: &str,
        params: Option<Value>,
    ) -> Result<T, String> {
        self.ensure_started()?;
        let id = self.next_id;
        self.next_id += 1;

        let mut request = json!({ "jsonrpc": "2.0", "id": id, "method": method });
        if let Some(params) = params {
            request["params"] = params;
        }

        let process = self
            .process
            .as_mut()
            .ok_or_else(|| "sidecar unavailable".to_string())?;

        process
            .stdin
            .write_all(format!("{}\n", request).as_bytes())
            .map_err(|e| format!("failed writing request: {}", e))?;
        process
            .stdin
            .flush()
            .map_err(|e| format!("failed flushing request: {}", e))?;

        let target_id = Value::from(id);
        let mut line = String::new();

        loop {
            line.clear();
            let n = process
                .stdout
                .read_line(&mut line)
                .map_err(|e| format!("failed reading response: {}", e))?;

            if n == 0 {
                let code = process.child.try_wait().ok().flatten();
                self.process = None;
                return Err(format!("sidecar exited unexpectedly: {:?}", code));
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let response: JsonRpcResponseEnvelope = match serde_json::from_str(trimmed) {
                Ok(r) => r,
                Err(_) => continue,
            };

            if response.id != target_id {
                continue;
            }

            if let Some(error) = response.error {
                let mut msg = format!("[{}] {}", error.code, error.message);
                if let Some(data) = error.data {
                    msg.push(' ');
                    msg.push_str(&data.to_string());
                }
                return Err(msg);
            }

            let value = response
                .result
                .ok_or_else(|| "missing result in response".to_string())?;
            return serde_json::from_value(value)
                .map_err(|e| format!("failed decoding sidecar response: {}", e));
        }
    }

    fn search_query(&mut self, query: &str) -> Result<SearchQueryResponse, String> {
        self.call("search.query", Some(json!({ "q": query })))
    }
}

struct SearchWindow {
    rpc: SidecarClient,
    text_input: Entity<TextInput>,
    results: Vec<SearchResult>,
    selected_result: Option<usize>,
    recent_searches: Vec<SharedString>,
    status: SharedString,
    backend: SharedString,
}

impl SearchWindow {
    fn query(&self, cx: &App) -> String {
        self.text_input.read(cx).text()
    }

    fn set_query(&mut self, query: String, cx: &mut Context<Self>) {
        self.text_input
            .update(cx, |input, cx| input.set_text(query, cx));
    }

    fn run_search(&mut self, cx: &mut Context<Self>) {
        let query = self.query(cx).trim().to_string();
        if query.is_empty() {
            self.status = "Type in search box first".into();
            return;
        }

        match self.rpc.search_query(&query) {
            Ok(response) => {
                self.results = response
                    .results
                    .into_iter()
                    .map(|item| SearchResult {
                        label: item.label.into(),
                        path: item.path.into(),
                        content: item
                            .content
                            .unwrap_or_else(|| "No preview available".into())
                            .into(),
                    })
                    .collect();

                let q: SharedString = query.clone().into();
                if !self.recent_searches.iter().any(|existing| existing == &q) {
                    self.recent_searches.insert(0, q);
                    self.recent_searches.truncate(10);
                }

                self.selected_result = None;
                self.status = format!("Found {} result(s)", self.results.len()).into();
            }
            Err(error) => {
                self.results.clear();
                self.selected_result = None;
                self.status = format!("Search failed: {}", error).into();
            }
        }
    }

    fn on_run_search(&mut self, _: &RunSearch, _window: &mut Window, cx: &mut Context<Self>) {
        self.run_search(cx);
        cx.notify();
    }

    fn on_search_click(&mut self, _: &MouseUpEvent, _window: &mut Window, cx: &mut Context<Self>) {
        self.run_search(cx);
        cx.notify();
    }

    fn on_minimize(&mut self, _: &MouseUpEvent, window: &mut Window, _: &mut Context<Self>) {
        window.minimize_window();
    }

    fn on_maximize(&mut self, _: &MouseUpEvent, window: &mut Window, _: &mut Context<Self>) {
        window.zoom_window();
    }

    fn on_close(&mut self, _: &MouseUpEvent, window: &mut Window, _: &mut Context<Self>) {
        window.remove_window();
    }
}

impl Render for SearchWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let selected = self.selected_result.and_then(|i| self.results.get(i));

        let left_list = if self.results.is_empty() {
            div().children(self.recent_searches.iter().map(|query| {
                let query_text = query.to_string();
                div()
                    .px_2()
                    .py_2()
                    .rounded_md()
                    .bg(rgb(0x18181b))
                    .hover(|d| d.bg(rgb(0x27272a)).cursor_pointer())
                    .child(div().text_sm().truncate().child(query.clone()))
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.set_query(query_text.clone(), cx);
                            this.run_search(cx);
                            cx.notify();
                        }),
                    )
            }))
        } else {
            div().children(self.results.iter().enumerate().map(|(index, result)| {
                let is_selected = self.selected_result == Some(index);
                div()
                    .px_2()
                    .py_2()
                    .rounded_md()
                    .bg(if is_selected {
                        rgb(0x3f3f46)
                    } else {
                        rgb(0x18181b)
                    })
                    .hover(|d| d.bg(rgb(0x27272a)).cursor_pointer())
                    .child(div().text_sm().truncate().child(result.path.clone()))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xa1a1aa))
                            .child(result.label.clone()),
                    )
                    .on_mouse_up(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            this.selected_result = Some(index);
                            cx.notify();
                        }),
                    )
            }))
        };

        div()
            .size_full()
            .bg(rgb(0x09090b))
            .text_color(rgb(0xe4e4e7))
            .flex()
            .flex_col()
            .on_action(cx.listener(Self::on_run_search))
            .child(
                div()
                    .h(px(38.0))
                    .w_full()
                    .bg(rgb(0x18181b))
                    .border_b_1()
                    .border_color(rgb(0x27272a))
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_3()
                    .child(
                        div()
                            .flex_1()
                            .window_control_area(WindowControlArea::Drag)
                            .text_sm()
                            .text_color(rgb(0xa1a1aa))
                            .child("the-search-thing"),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .rounded_sm()
                                    .bg(rgb(0x3f3f46))
                                    .window_control_area(WindowControlArea::Min)
                                    .hover(|d| d.bg(rgb(0x52525b)).cursor_pointer())
                                    .child("_")
                                    .on_mouse_up(MouseButton::Left, cx.listener(Self::on_minimize)),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .rounded_sm()
                                    .bg(rgb(0x3f3f46))
                                    .window_control_area(WindowControlArea::Max)
                                    .hover(|d| d.bg(rgb(0x52525b)).cursor_pointer())
                                    .child("□")
                                    .on_mouse_up(MouseButton::Left, cx.listener(Self::on_maximize)),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .rounded_sm()
                                    .bg(rgb(0x7f1d1d))
                                    .window_control_area(WindowControlArea::Close)
                                    .hover(|d| d.bg(rgb(0x991b1b)).cursor_pointer())
                                    .child("×")
                                    .on_mouse_up(MouseButton::Left, cx.listener(Self::on_close)),
                            ),
                    ),
            )
            .child(
                div()
                    .h(px(58.0))
                    .px_3()
                    .bg(rgb(0x111113))
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .flex_1()
                            .rounded_md()
                            .bg(rgb(0x27272a))
                            .border_1()
                            .border_color(rgb(0x3f3f46))
                            .child(self.text_input.clone()),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .bg(rgb(0x2563eb))
                            .hover(|d| d.bg(rgb(0x1d4ed8)).cursor_pointer())
                            .child("Search")
                            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_search_click)),
                    ),
            )
            .child(
                div().flex_1().min_h_0().p_3().child(
                    div()
                        .size_full()
                        .rounded_lg()
                        .border_1()
                        .border_color(rgb(0x3f3f46))
                        .bg(rgb(0x18181b))
                        .flex()
                        .overflow_hidden()
                        .child(
                            div()
                                .w(px(340.0))
                                .h_full()
                                .min_h_0()
                                .p_2()
                                .border_r_1()
                                .border_color(rgb(0x3f3f46))
                                .child(div().text_xs().text_color(rgb(0xa1a1aa)).child(
                                    if self.results.is_empty() {
                                        "Recent Searches"
                                    } else {
                                        "Results"
                                    },
                                ))
                                .child(left_list),
                        )
                        .child(div().flex_1().h_full().p_4().child(
                            if let Some(result) = selected {
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(0xa1a1aa))
                                            .child(result.path.clone()),
                                    )
                                    .child(
                                        div()
                                            .rounded_md()
                                            .bg(rgb(0x27272a))
                                            .p_3()
                                            .text_sm()
                                            .child(result.content.clone()),
                                    )
                            } else {
                                div()
                                    .size_full()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_color(rgb(0xa1a1aa))
                                    .child("Type in search box, click-select text, press Enter")
                            },
                        )),
                ),
            )
            .child(
                div()
                    .h(px(56.0))
                    .px_4()
                    .bg(rgb(0x111113))
                    .border_t_1()
                    .border_color(rgb(0x27272a))
                    .flex()
                    .items_center()
                    .justify_between()
                    .text_sm()
                    .text_color(rgb(0xa1a1aa))
                    .child(self.status.clone())
                    .child(self.backend.clone()),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.bind_keys([
            KeyBinding::new("backspace", Backspace, None),
            KeyBinding::new("delete", Delete, None),
            KeyBinding::new("left", Left, None),
            KeyBinding::new("right", Right, None),
            KeyBinding::new("shift-left", SelectLeft, None),
            KeyBinding::new("shift-right", SelectRight, None),
            KeyBinding::new("cmd-a", SelectAll, None),
            KeyBinding::new("ctrl-a", SelectAll, None),
            KeyBinding::new("cmd-v", Paste, None),
            KeyBinding::new("ctrl-v", Paste, None),
            KeyBinding::new("cmd-c", Copy, None),
            KeyBinding::new("ctrl-c", Copy, None),
            KeyBinding::new("cmd-x", Cut, None),
            KeyBinding::new("ctrl-x", Cut, None),
            KeyBinding::new("home", Home, None),
            KeyBinding::new("end", End, None),
            KeyBinding::new("ctrl-cmd-space", ShowCharacterPalette, None),
            KeyBinding::new("enter", RunSearch, None),
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("ctrl-q", Quit, None),
        ]);

        let bounds = Bounds::centered(None, size(px(1120.0), px(760.0)), cx);

        cx.open_window(
            WindowOptions {
                titlebar: None,
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| {
                let text_input = cx.new(|cx| TextInput::new(cx, "Search for files or folders…"));

                let app = cx.new(|_| SearchWindow {
                    rpc: SidecarClient::new(),
                    text_input: text_input.clone(),
                    results: vec![],
                    selected_result: None,
                    recent_searches: vec![],
                    status: "Ready".into(),
                    backend: "Not connected".into(),
                });

                window.focus(&text_input.read(cx).focus_handle(cx));

                cx.on_window_closed(|cx| cx.quit()).detach();
                cx.on_action(|_: &Quit, cx| cx.quit());

                app
            },
        )
        .unwrap();

        cx.activate(true);
        cx.observe_keystrokes(|event: &KeystrokeEvent, _, _| {
            if event.keystroke.key == "escape" {
                // no-op; kept to prove keystroke observation path is alive.
            }
        })
        .detach();
    });
}
