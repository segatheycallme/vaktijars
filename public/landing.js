let latitude, longitude;
const timezone = new Date().getTimezoneOffset() * -60;

// approx coords
fetch("https://ipwho.is?fields=latitude,longitude,city")
  .then((res) =>
    res.json().then((json) => {
      latitude = json.latitude;
      longitude = json.longitude;
      updatePosition();
    }),
  )
  .finally(() => {
    // exact coords
    navigator.geolocation.getCurrentPosition((pos) => {
      latitude = pos.coords.latitude;
      longitude = pos.coords.longitude;
      updatePosition();
    });
  });

function updatePosition() {
  htmx.find("#latitude").value = latitude;
  htmx.find("#longitude").value = longitude;
  htmx.find("#timezone").value = timezone;
  htmx.trigger("#vaktija", "update-vakat");
}

htmx.on("#active-search-form", "submit", async function (event) {
  event.preventDefault();

  // hate this
  while (htmx.find("#active-search-input").classList.contains("htmx-request")) {
    await new Promise((r) => setTimeout(r, 100));
  }

  const cities = htmx.find("#cities");
  latitude = cities.getAttribute("data-lat");
  longitude = cities.getAttribute("data-lon");

  updatePosition();
  htmx.swap("#cities", '<div id="cities"></div>', { swapStyle: "outerHTML" });
  htmx.find("#active-search-input").value = "";
});
