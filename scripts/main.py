from the_search_thing import walk_and_get_files_content

files_with_content = walk_and_get_files_content(
    "C:\\Users\\amaan\\OneDrive\\Documents\\coding\\the-search-thing\\src"
)
for path, content in files_with_content.items():
    print(f"{path}: {content}")
