import React from 'react';
import PropTypes from 'prop-types';
import debounce from 'lodash/debounce';

import Input from '../shared/input';
import { getCsrfToken } from '../helpers/authentication';
import Adjustments from './adjustments';

export default class Account extends React.Component {
  static propTypes = {
    id: PropTypes.number.isRequired,
    name: PropTypes.string.isRequired,
    balance: PropTypes.number.isRequired,
    debt: PropTypes.bool.isRequired,
    adjustments: PropTypes.arrayOf(PropTypes.object).isRequired,
  }

  constructor(props, context) {
    super(props, context);

    const { name, balance, debt } = this.props;
    this.state = { name, balance, debt };

    this.save = this.save.bind(this);
    this.updateState = this.updateState.bind(this);
  }

  save = debounce(() => {
    const headers = new Headers();

    headers.append('content-type', 'application/json');
    headers.append('x-csrf-token', getCsrfToken());

    fetch(`/api/accounts/${this.props.id}`, {
      method: 'PATCH',
      headers,
      credentials: 'same-origin',
      body: JSON.stringify({
        account: {
          name: this.state.name,
          balance: this.state.balance,
          debt: this.state.debt,
        },
      }),
    });
  }, 600);

  updateState(evt) {
    let value = null;
    const targetValue = evt.target.value;

    if (evt.target.type === 'checkbox') {
      value = evt.target.checked;
    } else {
      value = targetValue;
    }

    this.setState({
      [evt.target.name]: value,
    });

    this.save(evt);
  }

  render() {
    return (
      <div className="card">
        <div className="card-body">
          <h3 className="card-title">
            <Input type="text" name="name" required value={this.state.name} title="Account Name" callback={this.updateState} />
          </h3>
          <p>{this.state.debt ? 'Debt' : 'Credit'}</p>
          <form>
            <div>
              <div>
                <span>$</span>
              </div>
              <input type="number" name="balance" value={this.state.balance} onChange={this.updateState} />
            </div>
            <div>
              <label htmlFor="debt">
                <input name="debt" type="checkbox" checked={this.state.debt} onChange={this.updateState} />
                Debt
              </label>
            </div>
          </form>
          <Adjustments adjustments={this.props.adjustments} accountId={this.props.id} />
        </div>
      </div>
    );
  }
}
