import { afterAll, expect, setSystemTime, test } from "bun:test";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { mgmtUrl } from "./mgmt";

const saved = {
	files: process.env.LILYNK_ENV_FILES,
	url: process.env.LILYNK_MGMT_URL,
	token: process.env.LILYNK_MGMT_TOKEN,
};
afterAll(() => {
	setSystemTime();
	for (const [key, value] of [
		["LILYNK_ENV_FILES", saved.files],
		["LILYNK_MGMT_URL", saved.url],
		["LILYNK_MGMT_TOKEN", saved.token],
	] as const) {
		if (value === undefined) delete process.env[key];
		else process.env[key] = value;
	}
});

// The door's console starts once and the door may bind another port later.
test("the endpoint follows the console's own env file, and only the endpoint", () => {
	const file = join(mkdtempSync(join(tmpdir(), "pf-mgmt-")), "mgmt-endpoint");
	writeFileSync(
		file,
		"LILYNK_MGMT_URL=https://127.0.0.1:47995\nLILYNK_MGMT_TOKEN=other\n",
	);
	process.env.LILYNK_ENV_FILES = file;
	process.env.LILYNK_MGMT_URL = "https://127.0.0.1:47990";
	process.env.LILYNK_MGMT_TOKEN = "started-with";
	// Past the few seconds a read is kept for, whichever test asked last.
	setSystemTime(new Date(Date.now() + 60_000));
	expect(mgmtUrl()).toBe("https://127.0.0.1:47995");
	expect(process.env.LILYNK_MGMT_TOKEN).toBe("started-with");
});
