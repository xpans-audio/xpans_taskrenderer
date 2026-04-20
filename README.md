# xpans TaskRenderer
A rendering library dedicated to offline, non-realtime rendering of 
spatial audio scenes in the xpans Ecosystem

[![Crates.io Version](https://img.shields.io/crates/v/xpans_taskrenderer)](https://crates.io/crates/xpans_taskrenderer)
[![docs.rs](https://img.shields.io/docsrs/xpans_taskrenderer)](https://docs.rs/xpans_taskrenderer/0.1.0/xpans_taskrenderer/)

TaskRenderer uses the [Violet](https://github.com/xpans-audio/xpans_violet)
rendering engine and its extensions.

*Right now, this crate is a mess!*

## Tasks
Tasks are extended [render configurations](https://github.com/xpans-audio/xpans_renderconfig).

A render task in JSON would look like:
```json
{
  "name": "Headphones",
  "mode": "headphones",
  "config": {
    "pan_law": "sine",
    "max_itd_nanos": 650000,
    "distance_curve": "exponential",
    "distance_effect": 0.5,
    "min_distance": 0.1,
    "max_distance": 1.73
  }
}
```
The only difference between a task and a render configuration is the `name`
field, which usually indicates the file name of the render output.

Applications usually expect a *list* of tasks, so in JSON, you would
surround your tasks in square brackets, even if there is only one task:

```json
[
  {
    "name": "Headphones",
    "mode": "headphones",
    "config": {
      "pan_law": "sine",
      "max_itd_nanos": 650000,
      "distance_curve": "exponential",
      "distance_effect": 0.5,
      "min_distance": 0.1,
      "max_distance": 1.73
    }
  }
]
```
