import React from 'react';
import ReactDOM from 'react-dom';
import { BrowserRouter as Router, Route, Link } from 'react-router-dom';

import './index.scss';

import Accounts from './accounts/accounts';
import Goals from './goals/goals';
import Savings from './savings/savings';
import Home from './home';

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
        <Route exact path='/' component={Home}/>
        <Route exact path='/accounts' component={Accounts}/>
        <Route exact path='/savings' component={Savings}/>
        <Route exact path='/goals' component={Goals}/>
      </div>
    </div>
  </Router>,
  document.getElementById('main')
)
