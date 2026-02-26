
import os
import re

def fix_file(filepath):
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()

    # Regex to find .send_request(...) calls
    # We look for calls that have exactly 9 arguments (based on commas)
    # This is tricky due to nested parentheses, but for our code style it's mostly flat
    
    def add_tenth_arg(match):
        args_str = match.group(1)
        # Simple comma count (ignoring nested commas in json! macros for now as send_request args are usually simple)
        commas = args_str.count(',')
        if commas == 8: # 9 arguments have 8 commas
            # Check if it ends with a comma
            if args_str.strip().endswith(','):
                return f".send_request({args_str}None,)"
            else:
                return f".send_request({args_str}, None)"
        return match.group(0)

    # Use a more sophisticated regex to handle multi-line calls
    # This regex matches .send_request( followed by anything up to the balancing closing parenthesis
    # For our use case, we assume no complex nested calls inside the arguments
    new_content = re.sub(r'\.send_request\((.*?)\)', add_tenth_arg, content, flags=re.DOTALL)
    
    if new_content != content:
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(new_content)
        return True
    return False

src_dir = 'discord-privacy-util/src-tauri/src'
files_fixed = 0
for root, dirs, files in os.walk(src_dir):
    for file in files:
        if file.endswith('.rs'):
            if fix_file(os.path.join(root, file)):
                files_fixed += 1
                print(f"Fixed: {file}")

print(f"Total files fixed: {files_fixed}")
