import base64
import hashlib
from html.parser import HTMLParser
from pathlib import Path
import re
from zipfile import ZipFile

root = Path(__file__).resolve().parent.parent
dist = root / "dist"
html_bytes = (dist / "play.html").read_bytes()
html = html_bytes.decode("utf-8")


class EmbeddedAssets(HTMLParser):
    def handle_starttag(self, tag, attributes):
        attributes = dict(attributes)
        asset = attributes.get("src")
        if tag == "link":
            asset = attributes.get("href")
        if asset and not asset.startswith("data:"):
            raise ValueError(f"External asset in standalone HTML: {tag} {asset}")


EmbeddedAssets().feed(html)
for asset in re.findall(r"url\(([^)]+)\)", html):
    if not asset.startswith("data:"):
        raise ValueError(f"External CSS asset: {asset}")
match = re.search(r"load\('data:application/wasm;base64,([^']+)'\)", html)
if match is None:
    raise ValueError("Embedded WASM is missing")
wasm = base64.b64decode(match.group(1), validate=True)
if wasm[:8] != b"\x00asm\x01\x00\x00\x00":
    raise ValueError("Invalid WASM header")
if wasm != (dist / "until-dawn.wasm").read_bytes():
    raise ValueError("Embedded WASM differs from the built game")
version = hashlib.sha256(wasm).hexdigest()[:12]
if f"load('until-dawn.wasm?v={version}');" not in (dist / "index.html").read_text():
    raise ValueError("Hosted WASM URL does not contain the current build hash")
folder = "OddseedGames-LightUntilDawn"
with ZipFile(dist / f"{folder}.zip") as package:
    if package.testzip() is not None:
        raise ValueError("ZIP checksum verification failed")
    if package.read(f"{folder}/play.html") != html_bytes:
        raise ValueError("The archive contains a different game")
    for name in ["README.txt", "licenses/OFL.txt", "licenses/THIRD-PARTY-LICENSES.txt"]:
        if not package.read(f"{folder}/{name}"):
            raise ValueError(f"Missing package document: {name}")
print("Автономная сборка: WASM, встроенные ресурсы, ZIP и лицензии проверены")
