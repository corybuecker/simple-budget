import React from 'react';
import PropTypes from 'prop-types';
import merge from 'lodash/merge';
import debounce from 'lodash/debounce';
import cloneDeep from 'lodash/cloneDeep';

export default class Adjustment extends React.Component {
  static propTypes = {
    id: PropTypes.number.isRequired,
    total: PropTypes.number.isRequired,
    title: PropTypes.string.isRequired,
  }

  constructor(props, context) {
    super(props, context);

    this.state = cloneDeep(this.props);

    this.delete = this.delete.bind(this);
    this.updateInputValue = debounce(this.updateInputValue, 500);
    this.valueChanged = this.valueChanged.bind(this);
  }

  delete() {
    this.props.deleteAction(this.props.id);
  }

  valueChanged(e) {
    const newState = merge(this.state, {
      [e.target.name]: e.target.value
    });

    this.setState(newState);
    this.updateInputValue(newState);
  }

  updateInputValue(newState) {
    this.props.updateAction(newState);
  }

  render() {
    return (
      <tbody>
        <tr>
          <td>
            <input type='text' name='title' onChange={this.valueChanged} value={this.state.title} />
          </td>
          <td>
            <input type='text' name='total' onChange={this.valueChanged} value={this.state.total} />
          </td>
          <td>
            <button onClick={this.delete}>Delete</button>
          </td>
        </tr>
      </tbody>
    );
  }
}
