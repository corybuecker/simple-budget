import Loadable from 'react-loadable';
import React from 'react';

export default class LoadableGoals extends React.Component {
  constructor(){
    super();
    this.loadableComponent = Loadable({
      loader: () => import('./goals'),
      loading () {
        return <div>Loading...</div>
      }
    })
  }

  render() {
    return <this.loadableComponent />;
  }
}
