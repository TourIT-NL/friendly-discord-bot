
import os
import re

def fix_api_calls(content):
    def replace_standard(match):
        args = match.group(1)
        # Check if already 10 args by counting top-level commas
        comma_count = 0
        depth = 0
        for c in args:
            if c == '(': depth += 1
            elif c == ')': depth -= 1
            elif c == ',' and depth == 0: comma_count += 1
        
        if comma_count < 9:
            missing = 9 - comma_count
            new_args = args.strip()
            if not new_args.endswith(','): new_args += ','
            for _ in range(missing): new_args += " None,"
            return f".send_request({new_args})"
        return match.group(0)

    def replace_json(match):
        args = match.group(1)
        comma_count = 0
        depth = 0
        for c in args:
            if c == '(': depth += 1
            elif c == ')': depth -= 1
            elif c == ',' and depth == 0: comma_count += 1
        
        if comma_count < 5:
            missing = 5 - comma_count
            new_args = args.strip()
            if not new_args.endswith(','): new_args += ','
            for _ in range(missing): new_args += " None,"
            return f".send_request_json({new_args})"
        return match.group(0)

    content = re.sub(r'\.send_request\((.*?)\)', replace_standard, content, flags=re.DOTALL)
    content = re.sub(r'\.send_request_json\((.*?)\)', replace_json, content, flags=re.DOTALL)
    return content

src_dir = 'discord-privacy-util/src-tauri/src'
for root, dirs, files in os.walk(src_dir):
    for file in files:
        if file.endswith('.rs'):
            path = os.path.join(root, file)
            with open(path, 'r', encoding='utf-8') as f:
                old_content = f.read()
            new_content = fix_api_calls(old_content)
            if new_content != old_content:
                with open(path, 'w', encoding='utf-8') as f:
                    f.write(new_content)
                print(f"Fixed: {file}")
