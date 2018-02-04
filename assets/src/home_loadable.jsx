import Loadable from 'react-loadable';
import React from 'react';

export default class LoadableHome extends React.Component {
  constructor(){
    super();
    this.loadableComponent = Loadable({
      loader: () => import('./home'),
      loading () {
        return(
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
