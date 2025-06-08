const timezone = new Date().getTimezoneOffset() * -60;
// approx coords
fetch("https://ipwho.is?fields=latitude,longitude,city")
  .then((res) =>
    res.json().then((json) => {
      htmx.ajax("GET", "/vaktija", {
        values: {
          latitude: json.latitude,
          longitude: json.longitude,
          timezone: timezone,
        },
        target: "#vaktija",
        swap: "innerHTML",
      });
    }),
  )
  .finally(() => {
    // exact coords wip
    navigator.geolocation.getCurrentPosition((pos) => {
      htmx.ajax("GET", "/vaktija", {
        values: {
          latitude: pos.coords.latitude,
          longitude: pos.coords.longitude,
          timezone: timezone,
        },
        target: "#vaktija",
        swap: "innerHTML",
      });
    });
  });
