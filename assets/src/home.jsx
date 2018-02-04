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
      <div class="row">
        <div class="col-sm-6">
          <div class="card">
            <div class="card-header">
              Total Remaining
            </div>
            <div class="card-body">
              <p class="card-text">{this.state.daily.remaining}</p>
            </div>
          </div>
        </div>
        <div class="col-sm-6">
          <div class="card">
            <div class="card-header">
              Total Remaining per Day
            </div>
            <div class="card-body">
              <p class="card-text">{this.state.daily.remaining_per_day}</p>
            </div>
          </div>
        </div>
      </div>
    );
  }
}
