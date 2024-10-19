from luxeprint.luxeprint import rprint as print, render_table, highlight

# Data for the table
data = [
    ["[blue]Name", "[red]Age", "[green]City"],
    ["Alice", "30", "New York"],
    ["Bob", "25", "Los Angeles"],
    ["Charlie", "35", "Chicago"]
]

print("1. Render and print the table")
table_output = render_table(data)
print(table_output, end="\n\n")

print("2. Render and highlight python code")
code = '''
def greet(name):
    print(f"Hello, {name}!")

greet("World")
'''

# Highlight and print the code
highlighted_code = highlight(code, "python")
print(highlighted_code, end="\n\n")

print("3. Render and highlight rust code ")
code = '''
// Add any remaining text
if last_index < text.len() {
    let segment = &text[last_index..];
    let current_style = style_stack.last().cloned().unwrap_or_default();
    output.push(current_style.paint(segment));
}
'''
highlighted_code = highlight(code, "rust")
print(highlighted_code, end="\n\n")


print("4. Apply multiple styles")
print("[bold bright_red on_black underline]Hello, World!", end="\n\n")

print("4. Apply nested styles")
print("This is [i]very [red]important[/red] text[/i]. $120", end="\n\n")

print("5. Using emojis")
print("Hello, [bold magenta]World[/bold magenta]! :vampire:", end="\n\n")


# Example 4: Unknown emoji
print("6. Using unknown emojis")
print("Smile! :sport_utility_vehicle: :unknown_emoji: 120 or [red]822[/red]?", end="\n\n")
