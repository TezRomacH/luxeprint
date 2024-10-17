import ast

# Load the emoji mappings from the Python file
input_file = "_emoji_codes.py"  # Replace with your actual file name
output_file = "emojis.rs"

# Read the Python emoji mappings file
with open(input_file, "r", encoding="utf-8") as file:
    content = file.read()

# Parse the content to get the EMOJI dictionary
emoji_dict = ast.literal_eval(content.strip().split("=", 1)[1].strip())

# Create the Rust file and write the get_emoji function
with open(output_file, "w", encoding="utf-8") as rust_file:
    rust_file.write("// emojis.rs\n\n")
    rust_file.write("pub fn get_emoji(name: &str) -> &str {\n")
    rust_file.write("    match name {\n")

    for key, value in emoji_dict.items():
        rust_file.write(f"        \"{key}\" => \"{value}\",\n")

    rust_file.write("        _ => \":unknown_emoji:\",\n")
    rust_file.write("    }\n")
    rust_file.write("}\n")

print(f"{output_file} has been generated.")