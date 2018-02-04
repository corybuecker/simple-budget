import React from 'react';
import PropTypes from 'prop-types';
import find from 'lodash/find';
import chunk from 'lodash/chunk';
import without from 'lodash/without';

import { getCsrfToken } from '../helpers/authentication';
import Goal from './goal';
import './goals.scss';

export default class Goals extends React.Component {
  constructor(props, context) {
    super(props, context);
    this.state = {
      goals: [],
    };
    this.createGoal = this.createGoal.bind(this);
    this.deleteGoal = this.deleteGoal.bind(this);
    this.updateGoal = this.updateGoal.bind(this);
  }

  componentDidMount() {
    fetch('/api/goals', {credentials: 'same-origin'}).then(response => response.json()).then((json) => {
      this.setState({goals: json.data})
    })
  }

  children() {
    let goalGroups = chunk(this.state.goals, 2);

    return goalGroups.map(goals => (
      <div>
        { this.renderGoals(goals) }
      </div>
    ));
  }

  renderGoals(goals) {
    return goals.map(goal => (
      <Goal
        id={goal.id}
        title={goal.title}
        start_date={goal.start_date}
        end_date={goal.end_date}
        target={goal.target}
        deleteAction={this.deleteGoal}
        updateAction={this.updateGoal}
      />
    ));
  }

  deleteGoal(e) {
    const goalToDelete = find(this.state.goals, i => i.id === e);
    const newGoals = without(this.state.goals, goalToDelete);
    const url = `/api/goals/${goalToDelete.id}`;

    this.setState({ goals: newGoals });

    const headers = new Headers();

    headers.append('content-type', 'application/json');
    headers.append('x-csrf-token', getCsrfToken());

    fetch(url, {
      method: 'DELETE',
      headers,
      credentials: 'same-origin',
    });
  }

  updateGoal(goal) {
    let headers = new Headers();
    const url = `/api/goals/${goal.id}`;
    const method = 'PATCH';

    headers.append('Content-Type', 'application/json');
    headers.append('x-csrf-token', getCsrfToken());

    fetch(url, {
      method,
      headers,
      credentials: 'same-origin',
      body: JSON.stringify({
        goal: goal,
      }),
    });
  }

  createGoal() {
    let newGoals = this.state.goals.map(e => {
      Object.assign({}, e);
    });
    const url = '/api/goals';
    const headers = new Headers();

    headers.append('content-type', 'application/json');
    headers.append('x-csrf-token', getCsrfToken());

    fetch(url, {
      method: 'POST',
      headers,
      credentials: 'same-origin',
      body: JSON.stringify({
        goal: {
          title: 'goal',
          start_date: '2017-10-01',
          end_date: '2017-11-01',
          target: 100,
        }
      })
    }).then(response => {
      return response.json();
    }).then(data => {
      newGoals.push(data.data);
      this.setState({
        goals: newGoals
      });
    });
  }

  render() {
    return (
      <div>
        {this.children()}
        <button onClick={this.createGoal}>New</button>

      </div>
    );
  }
}
