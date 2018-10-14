module Accounts.Views exposing (editView, renderAccount, renderAccounts)

import Accounts.Messages exposing (..)
import Accounts.Models exposing (Account)
import Adjustments.Models exposing (Adjustment)
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Model exposing (Msg(..))


editView : Account -> Html Accounts.Messages.Msg
editView model =
    div []
        [ input [ type_ "text", value model.name, onInput Accounts.Messages.NameUpdated ] []
        , input [ type_ "checkbox", checked model.debt, onClick Accounts.Messages.ToggleDebt ] []
        , input [ type_ "text", value (String.fromFloat model.balance), onInput Accounts.Messages.BalanceUpdated ] []
        , button [ onClick Accounts.Messages.SaveAccount ] [ text "Save" ]
        , button [ onClick Accounts.Messages.DeleteAccount ] [ text "Delete" ]
        ]


renderAccounts : List Account -> Html Msg
renderAccounts accounts =
    div []
        [ button [ onClick CreateAccount ] [ text "New Account" ]
        , table []
            [ thead []
                [ tr []
                    [ th [] [ text "Account Name" ]
                    , th [] [ text "Balance" ]
                    , th [] [ text "Debt?" ]
                    ]
                ]
            , tbody [] (List.map renderAccount accounts)
            ]
        ]


renderAccount : Account -> Html Msg
renderAccount account =
    tr []
        (List.concat
            [ [ td [ onClick (OpenAccountEditor account) ] [ text account.name ]
              , td [] [ text (String.fromFloat account.balance) ]
              , td []
                    [ text
                        (if account.debt then
                            "True"

                         else
                            "False"
                        )
                    ]
              ]
            , List.map renderAdjustment account.adjustments
            ]
        )


renderAdjustment : Adjustment -> Html Msg
renderAdjustment model =
    tr []
        [ td [] [ text model.title ]
        , td [] [ text (String.fromFloat model.total) ]
        ]
