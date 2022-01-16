console.log("Executing main module in runtime %d", runtimeNum);

setTimeout(
  () => console.log("Timeout finished in runtime %d", runtimeNum),
  5000
);

try {
  const res = await fetch("https://deno.land");
  console.log("Fetch response in runtime %d", runtimeNum);
  const text = await res.text();
  console.log("Fetch stream finished in runtime %d", runtimeNum);
} catch (err) {
  console.log("Fetch failed in runtime %d:", runtimeNum, err);
}
