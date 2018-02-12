import React from 'react';
import PropTypes from 'prop-types';

import Account from './account';
import './accounts.scss';

export default class Accounts extends React.Component {
  constructor() {
    super();
    this.state = {accounts: []}
    this.children = this.children.bind(this);
  }

  componentDidMount() {
    fetch('/api/accounts', {credentials: 'same-origin'}).then(response => response.json()).then((json) => {
      this.setState({accounts: json.data})
    })
  }

  children() {
    return this.state.accounts.map(account => (
      <Account id={account.id} adjustments={account.adjustments} name={account.name} balance={account.balance} debt={account.debt} />
    ));
  }

  render() {
    return (
      <table className="table">
        {this.children()}
      </table>
    );
  }
}
