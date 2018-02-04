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
    path: path.join(__dirname, '../api/priv/static/js'),
    chunkFilename: "[name].[chunkhash].js",
    publicPath: "/js/"
  },
  plugins: [
    new CleanWebpackPlugin([path.join(__dirname, '../api/priv/static/js'), path.join(__dirname, '../api/priv/static/css')], { allowExternal: true }),
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
