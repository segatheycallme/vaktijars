# Vaktija.rs

Vaktija/prayer times dynamically calculated for every city in the world. \
_written in rust_

> [!WARNING]
> Don't trust this site to accurately calculate prayer times. No guarantees.

## API

There are two ways to use the public API:

- Querying via latitude and longitude

```http
GET /api/vaktija?timezone=7200&latitude=44.80&longitude=20.46
```

- Querying by city name

```http
GET /api/vaktija?timezone=7200&q=Belgrade
```

The `vakat` field in the response has the same order as the prayer times on the website:

```json
{
  "latitude": 44.8,
  "longitutde": 20.46,
  "city": "Belgrade",
  "timezone": 7200.0,
  "vakat": [
    "2025-06-21T02:20:35+02:00",
    "2025-06-21T04:52:18+02:00",
    "2025-06-21T12:40:02+02:00",
    "2025-06-21T16:48:55+02:00",
    "2025-06-21T20:27:45+02:00",
    "2025-06-21T22:46:27+02:00"
  ]
}
```

## Calculating prayer times

Most of the calculations are taken from <https://praytimes.org>.
Equation of time is adapted from Astronomical Algorithms and Calendrical Calculations.
For solar declination a simple approximation is used.

Displaying EoT and declination can be done using the eot.py and sd.py scripts.

The actual calculations are all done in the lib.rs file.

## Geographical data

To replicate the iconic <https://vaktija.ba> look,
displaying the names of cities was essential. Querying [geonames](https://download.geonames.org/)
or similar services would be too slow. They provide a generous 12mb dump of
cities with over 500 people, which trimmed down to only house a local name
and Geographical position data, takes up 700kb. Converting this into a R-Tree
takes some time, but has to only be done once at start up.
This allows us to quickly find the closest city of a given position.

Another potentially computation heavy thing is active search of cities.
I couldn't be bothered to find a efficient solution, so I used a smaller dataset
of cities over 15000 pop for the active search. The score of two strings is:
`levenshtein_distance(a,b) - a.starts_with(b) * 10`. This could be done way
better but it isn't a central functionality of the site.

## Displaying data

I used askama as a templating engine and HTMX to swap out old data.
One small challenge was updating the relative times every second without
bombarding the server with requests. HTMX wasn't really made for this use case,
but I worked out fine. Transporting variables using data- attributes made it easy
to enforce DRY principles. Only real challenge was the Serbian language.

## TODO

- [x] Public API
  - [ ] Query a specific day
- [ ] English language
- [ ] Efficient active search
