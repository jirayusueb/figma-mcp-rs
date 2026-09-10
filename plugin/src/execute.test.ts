import { expect, test, beforeEach } from "bun:test";
import { handleExecuteRequest } from "./execute";

beforeEach(() => {
  (globalThis as any).figma = {
    commitUndo: () => {},
  };
});
test("execute_code returns awaited value", async () => {
  const request = {
    type: "execute_code",
    requestId: "1",
    params: {
      code: "return 1 + 1",
    },
  };
  const result = await handleExecuteRequest(request);
  expect(result?.data.result).toBe(2);
  expect(result?.data.logs).toBeInstanceOf(Array);
});

test("execute_code serializes Figma node", async () => {
  const mockNode = {
    id: "123:456",
    name: "Test Node",
    type: "FRAME",
  };



  const request = {
    type: "execute_code",
    requestId: "1",
    params: {
      code: `return ${JSON.stringify(mockNode)}`,
    },
  };
  const result = await handleExecuteRequest(request);
  expect(result?.data.result).toEqual({
    id: "123:456",
    name: "Test Node",
    type: "FRAME",
  });
});

test("execute_code collects console.log output", async () => {


  const request = {
    type: "execute_code",
    requestId: "1",
    params: {
      code: `
        console.log("hello");
        console.warn("warning");
        console.error("error");
        return "done";
      `,
    },
  };
  const result = await handleExecuteRequest(request);
  expect(result?.data.result).toBe("done");
  expect(result?.data.logs.length).toBeGreaterThan(0);
  expect(result?.data.logs.some(l => l.includes("hello"))).toBe(true);
  expect(result?.data.logs.some(l => l.includes("warning"))).toBe(true);
  expect(result?.data.logs.some(l => l.includes("error"))).toBe(true);
});

test("execute_code rejects empty code", async () => {
  const request = {
    type: "execute_code",
    requestId: "1",
    params: {
      code: "   ",
    },
  };
  try {
    await handleExecuteRequest(request);
    expect.unreachable("should have thrown");
  } catch (error) {
    expect(error instanceof Error).toBe(true);
    expect((error as Error).message).toBe("code is required");
  }
});

test("execute_code passes through non-execute_code requests", async () => {
  const request = {
    type: "some_other_type",
    requestId: "1",
    params: {},
  };
  const result = await handleExecuteRequest(request);
  expect(result).toBe(null);
});
