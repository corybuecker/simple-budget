import Loadable from 'react-loadable';
import React from 'react';

export default class LoadableSavings extends React.Component {
  constructor(){
    super();
    this.loadableComponent = Loadable({
      loader: () => import('./savings'),
      loading () {
        return <div>Loading...</div>
      }
    })
  }

  render() {
    return <this.loadableComponent />;
  }
}
