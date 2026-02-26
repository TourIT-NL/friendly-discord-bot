
import json
import sys

try:
    with open('discord.com.har', 'r', encoding='utf-8') as f:
        har_data = json.load(f)

    entries = har_data.get('log', {}).get('entries', [])
    print(f"Total entries: {len(entries)}")

    headers_seen = set()
    for entry in entries[:20]:  # Look at first 20 entries
        request = entry.get('request', {})
        url = request.get('url', '')
        print(f"URL: {url}")
        for header in request.get('headers', []):
            name = header.get('name').lower()
            value = header.get('value')
            if name not in headers_seen:
                headers_seen.add(name)
                print(f"  Header: {name} = {value[:100]}...")
        print("-" * 20)

except Exception as e:
    print(f"Error: {e}")
