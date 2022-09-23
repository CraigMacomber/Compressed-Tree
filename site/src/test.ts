import { walkSubtree, WasmCursor, walkSubtreeInternal, walkSubtreeInternal2 } from "compressed-tree";

export {};

function fail(message: string): never {
  throw new Error(message);
}

async function doLoop(): Promise<void> {
  const fields = 1000;
  const nodes = 10;
  const expected = nodes * fields + 1;
  const cursor = new WasmCursor(fields, nodes);
  const outerRuns = 5;
  const runs = 1000;
  const walkers: [string, (w: WasmCursor) => number][] = [
    ["wasm", walkSubtree],
    ["wasm internal cursor", walkSubtreeInternal],
    ["wasm node", walkSubtreeInternal2],
    ["JS", walkSubtreeJS],
  ]

  const logger = document.getElementById('log') ?? fail("no log");
  function log(message: string): void {
    logger.innerHTML += message + '<br />';
  }

  for (const [name, walker] of walkers) {
    log(`${fields} of ${nodes}: (Total Nodes: ${expected}) ${name} walk`);
    for (let x = 1; x <= outerRuns; x++) {
      const t0 = performance.now();
      for (let i = 1; i <= runs; i++) {
        const count = walker(cursor);
        if (count !== expected) {
          throw new Error();
        }
      }
      const t1 = performance.now();
      const perRun = (t1 - t0) / runs;
      log(`${perRun.toFixed(3)} ms per run`);
    }
    log('');
    await new Promise(r => setTimeout(r, 0));
  }

  cursor.free();
  log("done");
}

if (typeof window !== "undefined") {
  (window as any)["doLoop"] = doLoop;
} else {
  it("cursor lifecycle", () => {
    new WasmCursor(0, 0).free();
  });
  it("cursor use", () => {
    const cursor = new WasmCursor(2, 5);
    const count = walkSubtreeJS(cursor);
    if (count !== 11) {
      throw new Error();
    }
    cursor.free();
  });
  it("cursor use wasm", () => {
    const cursor = new WasmCursor(2, 5);
    const count = walkSubtree(cursor);
    if (count !== 11) {
      throw new Error();
    }
    cursor.free();
  });
}

function walkSubtreeJS(n: WasmCursor): number {
  let count = 1;
  for (let inFields = n.firstField(); inFields; inFields = n.nextField()) {
    for (let inNodes = n.firstNode(); inNodes; inNodes = n.nextNode()) {
      count += walkSubtreeJS(n);
    }
  }
  return count;
}
