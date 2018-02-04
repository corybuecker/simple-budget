import React from 'react';

export default class Home extends React.Component {
  constructor(props, context) {
    super(props, context);
    this.state = {daily: {}}
  }

  componentDidMount() {
    fetch('/api/calculations', {credentials: 'same-origin'}).then(response => response.json()).then((json) => {
      this.setState({daily: json.data})
    })
  }

  render() {
    return (
      <div>
        {this.state.daily.remaining}
        <br />
        {this.state.daily.remaining_per_day}
      </div>
    );
  }
}
