# run-in-roblox Changelog

## Unreleased Changes
* **Breaking**: Reworked command line interface from the ground-up.
	* Places are now passed with `--place`.
	* Scripts are now passed via `--script`. A script is now always required.
* Added support for any place file, not just ones that rbx-dom supports.
* Fixed many panics, replacing them with graceful error messages.
* Switched from LogService capture to injection of stubs for print & warn, and use of xpcall to capture relevant logs only

## 0.2.0
* **TODO**

## 0.1.0
* Initial release