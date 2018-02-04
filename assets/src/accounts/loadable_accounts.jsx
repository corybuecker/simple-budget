import Loadable from 'react-loadable';
import React from 'react';

export default class LoadableAccounts extends React.Component {
  constructor(){
    super();
    this.loadableComponent = Loadable({
      loader: () => import('./accounts'),
      loading () {
        return (
          <div>
            Loading...
          </div>
        )
      }
    })
  }

  render() {
    return <this.loadableComponent />;
  }
}
