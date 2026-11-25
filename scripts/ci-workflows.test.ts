import { expect, test } from "bun:test";

test("PR tests use read-only permissions and match trusted push tests", async () => {
  const pr = Bun.YAML.parse(await Bun.file(".github/workflows/pr-ci.yml").text()) as any;
  const push = Bun.YAML.parse(await Bun.file(".github/workflows/publish.yml").text()) as any;
  expect(Object.keys(pr.on)).toEqual(["pull_request"]);
  expect(pr.permissions).toEqual({ contents: "read" });
  expect(Object.keys(push.on)).toEqual(["push"]);
  for (const name of ["test", "dotnet-test", "ruby-test", "jvm-test", "go-test"]) {
    const steps = pr.jobs[name].steps;
    for (const step of steps) {
      expect(step.with?.["allow-unsafe-pr-checkout"]).toBeUndefined();
      if (step.uses?.startsWith("actions/checkout@")) {
        expect(step.with?.["persist-credentials"]).toBe(false);
      }
    }
    const normalized = structuredClone(steps);
    for (const step of normalized) {
      if (step.uses?.startsWith("actions/checkout@")) delete step.with;
    }
    expect(normalized).toEqual(push.jobs[name].steps);
  }
});
