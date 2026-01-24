def main():
    print("Hello from the-search-thing!")


def add_function(x: int, y: int) -> int:
    from the_search_thing import add_numbers  # ty:ignore[unresolved-import]

    result = add_numbers(x, y)
    print(result)
    return result


if __name__ == "__main__":
    add_function(12, 13)
