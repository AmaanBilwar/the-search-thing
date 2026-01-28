def main():
    print("Hello from the-search-thing!")


def get_file_type_from_extension(file_extension: str) -> str:
    from the_search_thing import (
        get_file_type_with_extension,  # ty: ignore[unresolved-import]
    )

    return get_file_type_with_extension(file_extension)


def get_file_type(file_path: str) -> str:
    from the_search_thing import get_file_type  # ty: ignore[unresolved-import]

    return get_file_type(file_path)


def walk_and_get_content(directory: str):
    from the_search_thing import walk_and_get_content  # ty: ignore[unresolved-import]

    return walk_and_get_content(directory)


if __name__ == "__main__":
    import sys

    args = sys.argv[1:]
    if args:
        print(get_file_type(args[0]))
    else:
        print("Please provide a file path.")
        sys.exit(1)
