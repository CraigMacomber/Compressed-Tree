import * as compressed_tree from "compressed-tree";

export {};

async function doLoop(): Promise<void> {
  for (let i = 1; i <= 100; i++) {
    compressed_tree.greet("hi");
  }
  console.warn("done");
}

if (typeof window !== "undefined") {
  (window as any)["doLoop"] = doLoop;
} else {
  it("basic", () => {
    compressed_tree.greet("hi");
  });
}
