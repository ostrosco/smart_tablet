var addon = require('../native');
const {app, BrowserWindow} = require('electron');
const path = require('path');
const url = require('url');

function createWindow() {
    const mainWindow = new BrowserWindow({
        width: 800,
        height: 400,
        webPreferences: {
            nodeIntegration: true
        }
    });

    mainWindow.loadURL(
        url.format({
            pathname: path.join(__dirname, "../static/index.html"),
            protocol: "file:",
            slashes: true
        })
    );
}

app.whenReady().then(() => {
    app.allowRendererProcessReuse = false;
    createWindow();

    app.on('activate', function () {
        if (BrowserWindow.getAllWindows().length === 0) {
            createWindow();
        }
    });
});

app.on('window-all-closed', function () {
    if (process.platform !== 'darwin') {
        app.quit();
    }
});
