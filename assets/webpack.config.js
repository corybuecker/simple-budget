var path = require('path');

module.exports = {
  mode: 'development',
  watchOptions: {
    ignored: [/elm-stuff/, /node_modules/],
    poll: 1000
  },
  output: {
    filename: "main.js",
    path: path.resolve(__dirname, '../priv/static/js')
  },
  module: {
    rules: [{
        test: /\.html$/,
        exclude: /node_modules/,
        loader: 'file-loader?name=[name].[ext]'
      },
      {
        test: /\.elm$/,
        exclude: [/elm-stuff/, /node_modules/],
        // This is what you need in your own work
        loader: "elm-webpack-loader",

        options: {
          debug: true
        }
      }
    ]
  },

  devServer: {
    inline: true,
    stats: 'errors-only'
  }
};