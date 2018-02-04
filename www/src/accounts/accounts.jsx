import React from 'react';
import PropTypes from 'prop-types';
import chunk from 'lodash/chunk';

import Account from './account';
import './accounts.scss';

export default class Accounts extends React.Component {
  constructor() {
    super();
    this.state = {accounts: []}
    this.renderAccounts = this.renderAccounts.bind(this);
    this.children = this.children.bind(this);
  }

  componentDidMount() {
    fetch('/api/accounts', {credentials: 'same-origin'}).then(response => response.json()).then((json) => {
      this.setState({accounts: json.data})
    })
  }

  children() {
    let accountGroups = chunk(this.state.accounts, 2);

    return accountGroups.map(accounts => (
      <div>
        { this.renderAccounts(accounts) }
      </div>
    ));
  }

  renderAccounts(accounts) {
    return accounts.map(account => (
      <div>
        <Account key={account.id} id={account.id} adjustments={account.adjustments} name={account.name} balance={account.balance} debt={account.debt} />
      </div>
    ));
  }

  render() {
    return (
      <div>
        {this.children()}
      </div>
    );
  }
}
