

# Liithworks.js

An implementation of the Steamworks SDK for HTML/JS and NodeJS based applications.


## API

```js
const liithworks = require('liithworks.js')

// You can pass an appId, or don't pass anything and use a steam_appid.txt file
const client = liithworks.init(480)

// Print Steam username
console.log(client.localplayer.getName())

// Tries to activate an achievement
if (client.achievement.activate('ACHIEVEMENT')) {

}
```

You can refer to the [declarations file](https://github.com/tommyFromEarth/liithworks.js/blob/main/client.d.ts) to check the API support and get more detailed documentation of each function.

## Installation

To use Liithworks.js you don't have to build anything, just install it from npm:

```sh
$: npm i liithworks.js
```

### Electron

Liithworks.js is a native module and cannot be used by default in the renderer process. To enable the usage of native modules on the renderer process, the following configurations should be made on `main.js`:

```js
const mainWindow = new BrowserWindow({
    webPreferences: {
        contextIsolation: false,
        nodeIntegration: true
    }
})
```

To make the steam overlay working, call the `electronEnableSteamOverlay` on the end of your `main.js` file:

```js
require('Liithworks.js').electronEnableSteamOverlay()
```
 