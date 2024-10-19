# Project: luxeprint - Styled Printing and Tables in Rust for Python 🚀

👋 Welcome to `luxeprint`, a Rust 🦀 library for advanced styled printing, tables, and emoji rendering 😀. 

This project was created entirely by the 🤖 **ChatGPT o1-preview model** and is being maintained by someone with little to no experience in Rust 🦀. Please bear with me as we figure this out together! 🙏

## Install

```bash
pip intall -U luxeprint
```

## Examples of Use

Here are some examples of how to use luxeprint in your projects.

✨ Styled Printing

You can use the library to print styled text directly from your Python code:

```python
from luxeprint import rprint

# Print styled text 🖨️
rprint("Hello, [bold magenta]World[/bold magenta]! :vampire:")
```

📝 Render and Print Tables

Luxeprint also allows you to render and print tables, complete with ANSI styles:

```python
from luxeprint import render_table, rprint

# Data for the table 📊
data = [
    ["[blue]Name", "[red]Age", "[green]City"],
    ["Alice", "30", "New York"],
    ["Bob", "25", "Los Angeles"],
    ["Charlie", "35", "Chicago"]
]

# Render and print the table 🖨️
table_output = render_table(data)
rprint(table_output)
```

🎨 Highlight Code

Syntax highlighting is also available:

```python
from luxeprint import highlight

code = '''
def greet(name):
    print(f"Hello, {name}!")

greet("World")
'''

highlighted_code = highlight(code, "python")
rprint(highlighted_code)
```

🤝 How to Contribute

Since the maintainer (that's me!) doesn't know much about Rust 🦀, any help would be greatly appreciated 🙏. Here are some ways you can contribute:

1. 🐞 Bug Fixes: If you find any issues, please submit a Pull Request (PR) with a fix.

1. 📄 Documentation: Improving the documentation would help a lot, especially for those who aren't familiar with Rust.

1. ⚡ Optimization: If you know how to make the Rust code faster 🚀 or cleaner, feel free to contribute!

1. ✨ Adding Features: If there's a feature you think would be cool 😎, please add it and submit a PR.

To get started, simply fork the repository, make your changes, and submit a Pull Request. Since I'm still learning Rust 🦀, please keep your explanations simple! 🤓

📬 Contact

If you have questions ❓ or need clarification on anything, feel free to open an issue on the GitHub repository 🐙.
