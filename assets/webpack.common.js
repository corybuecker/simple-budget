const path = require('path');
const webpack = require('webpack');
const CleanWebpackPlugin = require('clean-webpack-plugin');
const glob = require('glob');

module.exports = {
  entry: {
    index: './src/index.jsx'
  },
  output: {
    filename: '[name].js',
    chunkFilename: '[name].[chunkhash].js',
    publicPath: "/js/",
    path: path.join(__dirname, '../priv/static/js'),
  },
  plugins: [
    new CleanWebpackPlugin([ path.join(__dirname, '../priv/static') ], { allowExternal: true }),

  ],
  resolve: {
    extensions: ['.js', '.jsx']
  },
  module: {
    rules: [{
      test: /\.scss$/,
      use: [
        { loader: 'style-loader' },
        {
          loader: 'css-loader',
          options: {
              modules: true,
              importLoaders: 1,
              sourceMap: true,
            },
        },
        {
          loader: 'sass-loader',
          options: {
            includePaths: [
              path.join(__dirname, "node_modules")
            ],
          },
        }
      ]},
      {
        test: /\.jsx?$/,
        use: [{
          loader: 'babel-loader',
        }],
        exclude: /node_modules/,
      },
    ],
  },
};
