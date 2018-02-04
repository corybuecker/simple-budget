import React from 'react';
import PropTypes from 'prop-types';
import Saving from './saving';
import './savings.scss';

export default class Savings extends React.Component {
  constructor(props, context) {
    super(props, context);
    this.state = {
      savings: [],
    };
    this.newSaving = this.newSaving.bind(this);
  }

  componentDidMount() {
    fetch('/api/savings', {credentials: 'same-origin'}).then(response => response.json()).then((json) => {
      this.setState({savings: json.data})
    })
  }

  children() {
    return this.state.savings.map(saving => (
      <div>
        <Saving key={saving.id} id={saving.id} title={saving.title} amount={saving.amount} />
      </div>
    ));
  }

  newSaving() {
    const newSavings = this.state.savings;
    newSavings.push({
      title: 'saving',
      id: null,
      amount: 10.0,
    });
    this.setState({
      savings: newSavings,
    });
  }

  render() {
    return (
      <div>
        <div>
          {this.children()}
        </div>
        <button onClick={this.newSaving} onKeyPress={this.newSaving} tabIndex='0'>New</button>
      </div>
    );
  }
}
