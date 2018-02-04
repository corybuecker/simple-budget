import React from 'react';
import PropTypes from 'prop-types';
import debounce from 'lodash/debounce';
import { getCsrfToken } from '../helpers/authentication';

export default class Goal extends React.Component {
  constructor(props, context) {
    super(props, context);
    this.state = {
      id: props.id,
      title: props.title,
      start_date: props.start_date,
      end_date: props.end_date,
      target: props.target,
    };
    this.handleChange = this.handleChange.bind(this);
    this.delete = this.delete.bind(this);
  }

  static propTypes = {
    title: PropTypes.string.isRequired,
    start: PropTypes.string.isRequired,
    end: PropTypes.string.isRequired,
    target: PropTypes.number.isRequired,
  }

  handleChange(event) {
    const target = event.target;
    const value = target.type === 'checkbox' ? target.checked : target.value;
    const name = target.name;

    this.setState({
      [name]: value,
    });

    this.update();
  }

  delete() {
    this.props.deleteAction(this.props.id);
  }

  componentDidUpdate() {
    this.props.updateAction(this.state);
  }

  render() {
    return (
      <div className="card">
        <div className='card-body'>
          <h3 className='card-title'>
            <input name='title' value={this.state.title} onChange={this.handleChange} />
          </h3>
          <form>
            <input name='target' value={this.state.target} onChange={this.handleChange} />
            <input type='date' name='start_date' value={this.state.start_date} onChange={this.handleChange} />
            <input type='date' name='end_date' value={this.state.end_date} onChange={this.handleChange} />
            <button onClick={this.delete}>Delete</button>
          </form>
        </div>
      </div>
    );
  }
}
