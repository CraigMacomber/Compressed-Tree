import { WasmCursor } from "compressed-tree";

export {};

async function doLoop(): Promise<void> {
  const cursor = new WasmCursor();
  for (let i = 1; i <= 100; i++) {
    const count = walkSubtree(cursor);
    if (count !== 11) {
      throw new Error();
    }
  }
  cursor.free();
  console.warn("done");
}

if (typeof window !== "undefined") {
  (window as any)["doLoop"] = doLoop;
} else {
  it("cursor lifecycle", () => {
    new WasmCursor().free();
  });
  it("cursor use", () => {
    const cursor = new WasmCursor();
    const count = walkSubtree(cursor);
    if (count !== 11) {
      throw new Error();
    }
    cursor.free();
  });
}

function walkSubtree(n: WasmCursor): number {
  let count = 1;
  for (let inFields = n.firstField(); inFields; inFields = n.nextField()) {
    for (let inNodes = n.firstNode(); inNodes; inNodes = n.nextNode()) {
      count += walkSubtree(n);
    }
  }
  return count;
}
