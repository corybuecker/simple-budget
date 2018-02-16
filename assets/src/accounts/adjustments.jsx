import React from 'react';
import PropTypes from 'prop-types';
import without from 'lodash/without';
import clone from 'lodash/clone';
import find from 'lodash/find';
import Adjustment from './adjustment';
import { getCsrfToken } from '../helpers/authentication';

export default class Adjustments extends React.Component {
  static propTypes = {
    accountId: PropTypes.number.isRequired,
    adjustments: PropTypes.array.isRequired,
  }

  constructor(props, context) {
    super(props, context);

    this.state = {
      adjustments: this.props.adjustments
    };

    this.deleteAdjustment = this.deleteAdjustment.bind(this);
    this.updateAdjustment = this.updateAdjustment.bind(this);
    this.createAdjustment = this.createAdjustment.bind(this);
  }

  children() {
    return this.state.adjustments.map(adjustment =>
      <Adjustment updateAction={this.updateAdjustment} deleteAction={this.deleteAdjustment}
        key={adjustment.id} id={adjustment.id} total={adjustment.total} title={adjustment.title} accountId={adjustment.account_id} />
    );
  }

  deleteAdjustment(e) {
    const adjustmentToDelete = find(this.state.adjustments, i => i.id === e);
    const newAdjustments = without(clone(this.state.adjustments), adjustmentToDelete);
    const url = `/api/accounts/${adjustmentToDelete.account_id}/adjustments/${adjustmentToDelete.id}`;

    this.setState({ adjustments: newAdjustments });

    const headers = new Headers();

    headers.append('content-type', 'application/json');
    headers.append('x-csrf-token', getCsrfToken());

    fetch(url, {
      method: 'DELETE',
      headers,
      credentials: 'same-origin',
    });
  }

  updateAdjustment(adjustment) {
    let headers = new Headers();
    const url = `/api/accounts/${this.props.accountId}/adjustments/${adjustment.id}`;
    const method = 'PATCH';

    headers.append('Content-Type', 'application/json');
    headers.append('x-csrf-token', getCsrfToken());

    fetch(url, {
      method,
      headers,
      credentials: 'same-origin',
      body: JSON.stringify({
        adjustment: adjustment,
      }),
    });
  }

  createAdjustment() {
    let newAdjustments = clone(this.state.adjustments);
    const url = `/api/accounts/${this.props.accountId}/adjustments`;
    const headers = new Headers();

    headers.append('content-type', 'application/json');
    headers.append('x-csrf-token', getCsrfToken());

    fetch(url, {
      method: 'POST',
      headers,
      credentials: 'same-origin',
      body: JSON.stringify({
        adjustment: {
          title: 'adjustment',
          total: 100,
        }
      })
    }).then(response => {
      return response.json();
    }).then(data => {
      newAdjustments.push(data.data);
      this.setState({
        adjustments: newAdjustments
      });
    }).catch(w => {
      console.log(w);
    });
  }

  render() {
    return (
      <table className="table">
        {this.children()}
        <button onClick={this.createAdjustment}>New</button>
      </table>
    );
  }
}
