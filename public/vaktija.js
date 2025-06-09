// i hate writing js

const timezone = new Date().getTimezoneOffset() * -60;

export function init() {
  updateTimer(document.getElementById("time"));
  setInterval(() => {
    updateTimer(document.getElementById("time"));
  }, 1000);
  for (const el of document.getElementsByClassName("kasno")) {
    updateSmallTime(el);
    setInterval(() => {
      updateSmallTime(el);
    }, 1000);
  }
}

function fxWidth(n) {
  const num = Math.trunc(n);
  if (num < 10) {
    return "0" + num;
  }
  return "" + num;
}
function updateTimer(el) {
  const next_prayer = Number(el.getAttribute("data-timestamp"));
  const secs = next_prayer - Date.now() / 1000 - timezone;
  if (secs < 0) {
    htmx.trigger("#vaktija", "update-vakat");
    return;
  }
  const hours = fxWidth(secs / 3600);
  const minutes = fxWidth((secs / 60) % 60);
  const seconds = fxWidth(secs % 60);
  el.textContent = `${hours}:${minutes}:${seconds}`;
}

function updateSmallTime(el) {
  const next_prayer = Number(el.getAttribute("data-timestamp"));
  let secs = next_prayer - Date.now() / 1000 - timezone;
  const prefix = secs > 0 ? "za" : "pre";
  secs = Math.abs(secs);

  let num;
  const hours = Math.trunc(secs / 3600);
  const minutes = Math.trunc((secs / 60) % 60);
  const seconds = Math.trunc(secs % 60);
  if (secs >= 3600) {
    num = hours;
  } else if (secs >= 60) {
    num = minutes;
  } else {
    num = seconds;
  }

  let unit;
  if (
    hours % 10 >= 5 ||
    (hours % 10 == 0 && hours > 0) ||
    (hours > 10 && hours < 20)
  ) {
    unit = "sati";
  } else if (hours % 20 > 1) {
    unit = "sata";
  } else if (hours % 20 == 1) {
    unit = "sat";
  } else if (
    minutes % 10 >= 5 ||
    (minutes % 10 == 0 && hours > 0) ||
    (minutes > 10 && minutes < 20)
  ) {
    unit = "minuta";
  } else if (minutes % 20 > 1) {
    unit = "minuta";
  } else if (minutes % 20 == 1) {
    unit = "minut";
  } else if (
    seconds % 10 >= 5 ||
    seconds % 10 == 0 ||
    (seconds > 10 && seconds < 20)
  ) {
    unit = "sekundi";
  } else if (seconds % 20 > 1) {
    unit = "sekunde";
  } else {
    unit = "sekundu";
  }

  el.textContent = `${prefix} ${num} ${unit}`;
}
