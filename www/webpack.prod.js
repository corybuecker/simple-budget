const merge = require('webpack-merge');
const common = require('./webpack.common.js');
const glob = require('glob');
const path = require('path');
const UglifyJSPlugin = require('uglifyjs-webpack-plugin');

module.exports = merge(common, {
  devtool: 'source-map',
  plugins: [
    new UglifyJSPlugin({
      cache: false,
      parallel: true,
      sourceMap: true,
    })
  ],
});
