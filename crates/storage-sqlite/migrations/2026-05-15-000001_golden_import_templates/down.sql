DELETE FROM import_templates
WHERE id IN (
    'system_yahoo_finance_holdings_golden',
    'system_yahoo_finance_transactions_golden',
    'system_ibkr_activity_golden',
    'system_fidelity_activity_golden',
    'system_schwab_activity_golden',
    'system_generic_bank_golden',
    'system_fixed_deposit_golden',
    'system_private_capital_call_golden',
    'system_fixed_income_cashflow_golden'
);
