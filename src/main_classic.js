console.log("Executing main module in runtime %d", runtimeNum);

setTimeout(
  () => console.log("Timeout finished in runtime %d", runtimeNum),
  2000
);

fetch("https://deno.land")
  .then((res) => {
    console.log("Fetch response in runtime %d", runtimeNum);
    return res.text();
  })
  .then(() => {
    console.log("Fetch stream finished in runtime %d", runtimeNum);
  })
  .catch((err) => {
    console.log("Fetch failed in runtime %d:", runtimeNum, err);
  });
