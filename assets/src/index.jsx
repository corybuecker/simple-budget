import React from 'react';
import ReactDOM from 'react-dom';
import { BrowserRouter as Router, Route, Link } from 'react-router-dom';

import './index.scss';

import LoadableAccounts from './accounts/loadable_accounts';
import LoadableGoals from './goals/loadable_goals';
import LoadableSavings from './savings/loadable_savings';
import LoadableHome from './home_loadable';

ReactDOM.render(
  <Router>
    <div className='container'>
      <nav className='nav justify-content-center'>
        <Link className='nav-link' to='/'>Balances</Link>
        <Link className='nav-link' to='/accounts'>Accounts</Link>
        <Link className='nav-link' to='/savings'>Savings</Link>
        <Link className='nav-link' to='/goals'>Goals</Link>
      </nav>
      <div>
        <Route exact path='/' component={LoadableHome}/>
        <Route exact path='/accounts' component={LoadableAccounts}/>
        <Route exact path='/savings' component={LoadableSavings}/>
        <Route exact path='/goals' component={LoadableGoals}/>
      </div>
    </div>
  </Router>,
  document.getElementById('main')
)
