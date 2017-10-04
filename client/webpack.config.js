const path = require('path');

const phaserRoot = path.join(__dirname, 'node_modules/phaser/build/custom/');


module.exports = {
    devtool: 'inline-source-map',
    entry: './src/index.ts',
    output: {
        filename: 'bundle.js',
    },
    resolve: {
        extensions: ['.ts', '.tsx', '.js'],
        alias: {
            phaser: path.join(phaserRoot, 'phaser-split.js'),
            pixi: path.join(phaserRoot, 'pixi.js'),
            p2: path.join(phaserRoot, 'p2.js'),
        }
    },
    externals: {
        Phaser: "Phaser",
    },
    module: {
        loaders: [
            { test: /pixi.js/, loader: 'script-loader' },
            { test: /p2.js/, loader: 'script-loader' },
            { test: /phaser-split.js/, loader: 'script-loader' },
            { test: /\.tsx?$/, loader: 'ts-loader' },
        ]
    }
}
