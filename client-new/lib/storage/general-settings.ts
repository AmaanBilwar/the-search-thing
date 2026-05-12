export type ThemeSetting = "dark" | "light" | "gruvbox-dark" | "gruvbox-light";
export type SearchScopeSetting = "both" | "files" | "folders";
export type FontSetting = "sans-serif" | "mono";
export type WindowPlacementSetting = "center" | "center-above" | "center-below" | "cursor";
type GeneralSettingKey =
  | "launch-on-startup"
  | "theme"
  | "font"
  | "scope"
  | "window-placement"
  | "clear-search";

export type GeneralSettingsState = {
  "launch-on-startup": boolean;
  theme: ThemeSetting;
  font: FontSetting;
  scope: SearchScopeSetting;
  "window-placement": WindowPlacementSetting;
};

export const DEFAULT_GENERAL_SETTINGS: GeneralSettingsState = {
  "launch-on-startup": true,
  theme: "dark",
  font: "sans-serif",
  scope: "both",
  "window-placement": "center",
};

/** Dark UI variants (Tailwind `dark:` + color-scheme). */
export function themeIsDark(theme: ThemeSetting): boolean {
  return theme === "dark" || theme === "gruvbox-dark";
}

export const GENERAL_SETTINGS_CHANGE_EVENT = "general-settings:change";

// current general settings actions
type GeneralMeta = {
  action: GeneralSettingKey;
  label: string;
  description: string;
};

// all keybind actions right now
const SETTINGS_ACTIONS: GeneralMeta[] = [
  {
    action: "launch-on-startup",
    label: "Launch at startup",
    description: "Open the app when you sign in.",
  },
  {
    action: "theme",
    label: "Theme",
    description: "Choose appearance (Ayu or Gruvbox, dark or light).",
  },
  { action: "font", label: "Font", description: "Choose Sans-Serif or Mono." },
  // { action: "scope", label: "Search Scope", description: "Files, Folders, or Both." },
  {
    action: "window-placement",
    label: "Window placement",
    description: "Choose where the window appears.",
  },
  {
    action: "clear-search",
    label: "Clear recent searches",
    description: "Remove cached query history.",
  },
];
