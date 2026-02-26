
import os
import re

def count_args(s):
    # Very simple argument counter that handles balanced parentheses
    count = 0
    depth = 0
    for char in s:
        if char == '(':
            depth += 1
        elif char == ')':
            depth -= 1
        elif char == ',' and depth == 0:
            count += 1
    # Number of args is count of commas + 1, unless it's empty
    if not s.strip():
        return 0
    return count + 1

def fix_file(filepath):
    with open(filepath, 'r', encoding='utf-8') as f:
        content = f.read()

    def replace_call(match):
        method_name = match.group(1)
        args_str = match.group(2)
        
        num_args = count_args(args_str)
        
        if method_name == "send_request":
            target = 10
        elif method_name == "send_request_json":
            target = 6
        else:
            return match.group(0)
            
        if num_args < target:
            missing = target - num_args
            new_args = args_str.strip()
            if not new_args.endswith(','):
                new_args += ','
            
            for _ in range(missing):
                new_args += " None,"
            
            return f".{method_name}({new_args})"
        return match.group(0)

    # Regex to match .method_name(args)
    # We use a non-greedy match for the content, but we need to be careful with nested parens.
    # For simplicity, we'll assume the calls don't have extremely complex nesting.
    new_content = re.sub(r'\.(send_request(?:_json)?)\((.*?)\)', replace_call, content, flags=re.DOTALL)
    
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
