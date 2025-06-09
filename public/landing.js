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

export function updatePosition() {
  htmx.find("#latitude").value = latitude;
  htmx.find("#longitude").value = longitude;
  htmx.find("#timezone").value = timezone;
  htmx.trigger("#vaktija", "update-vakat");
}
