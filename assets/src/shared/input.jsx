import React from 'react';
import PropTypes from 'prop-types';

export default class Input extends React.Component {
  static propTypes = {
    title: PropTypes.string.isRequired,
    name: PropTypes.string.isRequired,
    type: PropTypes.string.isRequired,
    value: PropTypes.oneOfType([
      PropTypes.string,
      PropTypes.number,
    ]),
    required: PropTypes.bool.isRequired,
    callback: PropTypes.func.isRequired,
  }

  constructor(props, context) {
    super(props, context);

    this.state = { value: this.props.value, editing: false };

    this.handleChange = this.handleChange.bind(this);
    this.handleBlur = this.handleBlur.bind(this)
    this.setEditing = this.setEditing.bind(this)
  }

  handleChange(e) {
    this.setState({value: e.target.value});
  }

  handleBlur(e) {
    this.setState({editing: false})
    this.props.callback(e);
  }

  setEditing() {
    this.setState({editing: true});

  }

  render() {
    let output = null;
    if (this.state.editing) {
      output = <input type={this.props.type} name={this.props.name} value={this.state.value} onBlur={this.handleBlur} onChange={this.handleChange} />
    } else {
      output = <span onClick={this.setEditing}>{this.state.value}</span>;
    }
    return output;
  }
}
