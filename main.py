def main():
    print("Hello from the-search-thing!")


def walk_and_get_content(directory: str):
    from the_search_thing import walk_and_get_content  # ty: ignore[unresolved-import]

    return walk_and_get_content(directory)


if __name__ == "__main__":
    import sys

    args = sys.argv[1:]
    if args:
        walk_and_get_content(args[0])
    else:
        print("Please provide a directory path.")
        sys.exit(1)
