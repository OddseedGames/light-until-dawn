import base64
import hashlib
from pathlib import Path
import re
from zipfile import ZIP_DEFLATED, ZipFile

root = Path(__file__).resolve().parent.parent
dist = root / "dist"
html = (dist / "index.html").read_text()
css = (dist / "style.css").read_text()
font = base64.b64encode((dist / "IBMPlexMono-Regular.ttf").read_bytes()).decode()
wasm_bytes = (dist / "until-dawn.wasm").read_bytes()
wasm = base64.b64encode(wasm_bytes).decode()
version = hashlib.sha256(wasm_bytes).hexdigest()[:12]
wasm_url = f"until-dawn.wasm?v={version}"
html, replacements = re.subn(
    r"load\('until-dawn\.wasm(?:\?v=[a-f0-9]+)?'\);",
    f"load('{wasm_url}');", html,
)
if replacements != 1:
    raise ValueError("Expected one WASM loader in index.html")
(dist / "index.html").write_text(html)
loader = (dist / "mq_js_bundle.js").read_text()
css = css.replace("url(IBMPlexMono-Regular.ttf)", f"url(data:font/ttf;base64,{font})")
html = html.replace('<link rel="stylesheet" href="style.css">', f"<style>{css}</style>")
html = html.replace('<script src="mq_js_bundle.js"></script>', f"<script>{loader}</script>")
html = html.replace(f"load('{wasm_url}');", f"load('data:application/wasm;base64,{wasm}');")
(dist / "play.html").write_text(html)
print(f"Автономная игра: {dist / 'play.html'}")

readme = (root / "docs" / "QUICKSTART.txt").read_text(encoding="utf-8")
archive = dist / "OddseedGames-LightUntilDawn.zip"
folder = "OddseedGames-LightUntilDawn"
with ZipFile(archive, "w", compression=ZIP_DEFLATED, compresslevel=9) as package:
    package.write(dist / "play.html", f"{folder}/play.html")
    package.writestr(f"{folder}/README.txt", readme.encode("utf-8-sig"))
    for license_name in ["THIRD-PARTY-LICENSES.txt", "OFL.txt"]:
        package.write(dist / license_name, f"{folder}/licenses/{license_name}")
print(f"Архив для отправки: {archive} ({archive.stat().st_size / 1024:.0f} КБ)")
