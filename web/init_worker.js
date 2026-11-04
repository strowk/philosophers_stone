import init, { wasm_start_worker } from "./philosopher_stone.js";
console.log("Running worker init");
await init();
console.log("Running worker start");
await wasm_start_worker();
console.log("Worker started");