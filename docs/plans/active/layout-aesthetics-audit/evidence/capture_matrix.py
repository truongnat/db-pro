import subprocess, os, struct, hashlib, json
from pathlib import Path

repo = Path('/Users/truongdq/tx/GitHub/db-pro')
ev = Path(__file__).resolve().parent
ev.mkdir(parents=True, exist_ok=True)
data = Path('/private/tmp/dbpro-audit/data')
data.mkdir(parents=True, exist_ok=True)

surfaces = {
    'welcome': {'DB_PRO_CAPTURE_WELCOME': '1'},
    'query': {'DB_PRO_CAPTURE_QUERY': '1'},
    'table': {'DB_PRO_CAPTURE_TABLE': '1'},
    'explorer': {
        'DB_PRO_CAPTURE_FILTER': '1',
        'DB_PRO_CAPTURE_REASSERT_SURFACE': '1',
        'DB_PRO_CAPTURE_EXPLORER_FILTER_POPOVER': '1',
    },
    'diagram': {'DB_PRO_CAPTURE_DIAGRAM': '1'},
    'settings': {'DB_PRO_CAPTURE_FEATURE': 'settings-general'},
    'connection': {'DB_PRO_CAPTURE_NEW_CONNECTION': '1'},
    'gallery': {'DB_PRO_CAPTURE_COMPONENT_GALLERY': '1'},
    'agent': {'DB_PRO_CAPTURE_AGENT': '1'},
    'palette': {'DB_PRO_CAPTURE_QUICK_OPEN': '1'},
}

light_env = {
    'welcome': ('DB_PRO_CAPTURE_WELCOME_LIGHT', '1'),
    'query': ('DB_PRO_CAPTURE_QUERY_LIGHT', '1'),
    'table': ('DB_PRO_CAPTURE_TABLE_LIGHT', '1'),
    'explorer': ('DB_PRO_CAPTURE_EXPLORER_FILTER_LIGHT', '1'),
    'settings': ('DB_PRO_CAPTURE_FEATURE_LIGHT', '1'),
    'connection': ('DB_PRO_CAPTURE_CONNECTION_LIGHT', '1'),
    'palette': ('DB_PRO_CAPTURE_QUICK_OPEN_LIGHT', '1'),
    'diagram': ('DB_PRO_CAPTURE_DIAGRAM_LIGHT', '1'),
    'gallery': ('DB_PRO_CAPTURE_GALLERY_LIGHT', '1'),
    'agent': ('DB_PRO_CAPTURE_AGENT_LIGHT', '1'),
}

rows = []
for name, extra in surfaces.items():
    for size in ['1280x800', '1440x900', '1920x1080']:
        for theme in ['light', 'dark']:
            cell = f'{name}-{theme}-{size}'
            png = ev / f'{cell}.png'
            env = os.environ.copy()
            env.update({
                'DB_PRO_DATA_DIR': str(data),
                'DB_PRO_DISABLE_KEYRING': '1',
                'DB_PRO_WINDOW_SIZE': size,
                'DB_PRO_CAPTURE_TO': str(png),
                'DB_PRO_CAPTURE_SETTLE_FRAMES': '45',
                # Without this the opener runs once and a late schema-load event
                # restores the Welcome tab, so the cell silently captures the wrong
                # surface while still reporting png:true.
                'DB_PRO_CAPTURE_REASSERT_SURFACE': '1',
                **extra,
            })
            if theme == 'light' and name in light_env:
                key, value = light_env[name]
                env[key] = value
            try:
                result = subprocess.run(
                    ['./target/debug/db-pro-native'], cwd=str(repo), env=env,
                    capture_output=True, timeout=45,
                )
                code = result.returncode
            except subprocess.TimeoutExpired:
                code = 124
            entry = {
                'cell': cell, 'surface': name, 'theme': theme,
                'requested': size, 'exit': code, 'png': png.exists(),
            }
            if png.exists():
                blob = png.read_bytes()
                w, h = struct.unpack('>II', blob[16:24])
                entry.update({
                    'rendered': [w / 2, h / 2],
                    'sha256': hashlib.sha256(blob).hexdigest()[:16],
                })
            rows.append(entry)
            print(f"{cell:34} exit={code:4} {entry.get('rendered')}", flush=True)

(ev / 'manifest.json').write_text(json.dumps(rows, indent=2))
print('done', sum(r['png'] for r in rows), 'of', len(rows))
