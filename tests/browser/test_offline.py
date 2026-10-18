from pathlib import Path
import unittest

from playwright.sync_api import expect, sync_playwright


ROOT = Path(__file__).resolve().parents[2]


class OfflineGameTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.playwright = sync_playwright().start()
        cls.browser = cls.playwright.chromium.launch(
            args=["--use-angle=swiftshader", "--enable-unsafe-swiftshader"]
        )

    @classmethod
    def tearDownClass(cls):
        cls.browser.close()
        cls.playwright.stop()

    def setUp(self):
        self.context = self.browser.new_context(
            viewport={"width": 1440, "height": 1100}, offline=True
        )
        self.context.tracing.start(screenshots=True, snapshots=True, sources=True)
        self.page = self.context.new_page()
        self.errors = []
        self.external_requests = []
        self.page.on("pageerror", lambda error: self.errors.append(str(error)))
        self.page.on(
            "console",
            lambda message: self.errors.append(message.text) if message.type == "error" else None,
        )
        self.page.on(
            "request",
            lambda request: self.external_requests.append(request.url)
            if request.url.startswith(("http://", "https://")) else None,
        )
        self.page.goto((ROOT / "dist/play.html").as_uri())
        self.status = self.page.locator("#game-status")
        expect(self.status).to_have_attribute("data-phase", "0", timeout=30_000)
        expect(self.page.locator("#loading")).to_be_hidden()
        expect(self.page.locator("#load-error")).to_be_hidden()
        self.page.locator("canvas").focus()

    def tearDown(self):
        diagnostics = ROOT / "test-results" / self.id().rsplit(".", 1)[-1]
        diagnostics.mkdir(parents=True, exist_ok=True)
        try:
            self.page.screenshot(path=str(diagnostics / "screen.png"), full_page=True)
            self.context.tracing.stop(path=str(diagnostics / "trace.zip"))
            self.assertEqual(self.errors, [])
            self.assertEqual(self.external_requests, [])
        finally:
            self.context.close()

    def state(self):
        return self.status.evaluate(
            "element => Object.fromEntries(Object.entries(element.dataset)"
            ".map(([key, value]) => [key, Number(value)]))"
        )

    def wait_for(self, condition):
        self.page.wait_for_function(
            "() => { const s = Object.fromEntries(Object.entries("
            "document.querySelector('#game-status').dataset)"
            ".map(([key, value]) => [key, Number(value)])); return " + condition + "; }",
            timeout=20_000,
        )

    def click_game(self, x, y):
        canvas = self.page.locator("canvas")
        box = canvas.bounding_box()
        canvas.click(position={"x": x / 1280 * box["width"], "y": y / 900 * box["height"]})

    def test_difficulty_and_restart(self):
        for mode, fuel in [(2, 38), (0, 54), (1, 42)]:
            self.page.keyboard.press("q")
            expect(self.status).to_have_attribute("data-difficulty", str(mode))
            self.assertEqual(self.state()["fuel"], fuel)
        self.page.keyboard.press("Enter")
        expect(self.status).to_have_attribute("data-phase", "1")
        self.page.keyboard.press("p")
        expect(self.status).to_have_attribute("data-phase", "2")
        self.page.keyboard.press("r")
        expect(self.status).to_have_attribute("data-phase", "1")
        self.assertEqual(self.state()["searchesLeft"], 3)

    def test_fuel_trip_movement_sprint_and_pause(self):
        self.page.keyboard.press("Enter")
        expect(self.status).to_have_attribute("data-phase", "1")
        self.page.keyboard.press("2")
        self.page.keyboard.press("3")
        self.page.keyboard.press("e")
        self.wait_for("s.outsideSeconds > 0")
        self.page.keyboard.down("Shift")
        self.page.keyboard.down("a")
        self.wait_for("s.position > 0.2")
        self.page.keyboard.up("a")
        self.page.keyboard.up("Shift")
        self.assertLess(self.state()["stamina"], 95)
        self.click_game(976, 748)
        self.wait_for("s.position > 0.525 && s.position < 0.58")
        self.page.keyboard.press("e")
        self.wait_for("s.carrying === 1")
        self.page.keyboard.press("p")
        expect(self.status).to_have_attribute("data-phase", "2")
        paused = self.state()
        self.page.wait_for_timeout(500)
        self.assertEqual(self.state(), paused)
        self.page.keyboard.press("Enter")
        expect(self.status).to_have_attribute("data-phase", "1")
        self.click_game(1084, 748)
        self.wait_for("s.position < 0.035")
        fuel = self.state()["fuel"]
        self.page.keyboard.press("e")
        self.wait_for("s.outsideSeconds === 0 && s.searchesLeft === 2")
        self.assertGreater(self.state()["fuel"], fuel + 16)


if __name__ == "__main__":
    unittest.main()
