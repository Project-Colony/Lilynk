// Where each patch attempt got to, readable from the CEF debugger as `window.__lilynkDiag`
// and echoed to the console. Steam's tree is not an API; when it moves, this says which step.
import { read } from "./storage";

declare global {
  interface Window {
    __lilynkDiag?: string[];
  }
}

let lastDiag = "";

/** Storage key `lilynk:diagVerbose` set to "1" traces every render step instead of changes. */
export function verbose(): boolean {
  return read("lilynk:diagVerbose") === "1";
}

export function diag(msg: string): void {
  if (msg === lastDiag && !verbose()) {
    return; // the page renders several times per open; one line per change is enough
  }
  lastDiag = msg;
  const line = `${new Date().toISOString().slice(11, 19)} ${msg}`;
  console.warn(`lilynk: ${msg}`);
  try {
    (window.__lilynkDiag ??= []).push(line);
    if (window.__lilynkDiag.length > 60) {
      window.__lilynkDiag.shift();
    }
  } catch {
    /* ignore */
  }
}
