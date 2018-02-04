import React from 'react';
import PropTypes from 'prop-types';
import debounce from 'lodash/debounce';
import { getCsrfToken } from '../helpers/authentication';
import styles from './savings.scss';

export default class Goal extends React.Component {
  save = debounce((e) => {
    const headers = new Headers();
    headers.append('Content-Type', 'application/json');
    let method = 'PATCH';
    headers.append('x-csrf-token', getCsrfToken());
    let url = `/api/savings/${this.props.id}`;
    if (!this.props.id) {
      method = 'POST';
      url = '/api/savings';
    }
    fetch(url, {
      method,
      headers,
      credentials: 'same-origin',
      body: JSON.stringify({
        saving: this.state,
      }),
    });
  }, 600)

  static propTypes = {
    title: PropTypes.string.isRequired,
    amount: PropTypes.number.isRequired,
  }

  constructor(props, context) {
    super(props, context);
    this.state = {
      title: props.title,
      amount: props.amount,
    };
    this.handleChange = this.handleChange.bind(this);
    this.save = this.save.bind(this);
    this.delete = this.delete.bind(this);
  }

  handleChange(event) {
    const target = event.target;
    const value = target.type === 'checkbox' ? target.checked : target.value;
    const name = target.name;

    this.setState({
      [name]: value,
    });
    this.save();
  }

  delete() {
    const headers = new Headers();

    const url = `/api/savings/${this.props.id}`;
    headers.append('Content-Type', 'application/json');
    headers.append('x-csrf-token', getCsrfToken());

    fetch(url, {
      method: 'DELETE',
      headers,
      credentials: 'same-origin',
    });
  }

  render() {
    return (
      <div className={[styles.card, `card`].join(' ')}>
        <div className='card-body'>
          <h3 className='card-title'>
            <input name='title' value={this.state.title} onChange={this.handleChange} />
          </h3>
          <form>
            <input name='amount' value={this.state.amount} onChange={this.handleChange} />
            <button onClick={this.delete}>Delete</button>
          </form>
        </div>
      </div>
    );
  }
}
