module Accounts.Views exposing (editView, renderAccount, renderAccounts)

import Accounts.Messages
import Accounts.Models exposing (Account)
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Model exposing (Msg(..))


editView : Account -> Html Accounts.Messages.Msg
editView model =
    div []
        [ input [ type_ "text", value model.name, onInput Accounts.Messages.NameUpdated ] []
        , input [ type_ "checkbox", checked model.debt ] []
        , input [ type_ "text", value (String.fromFloat model.balance) ] []
        ]


renderAccounts : List Account -> Html Msg
renderAccounts accounts =
    table []
        [ thead []
            [ tr []
                [ th [] [ text "Account Name" ]
                , th [] [ text "Balance" ]
                , th [] [ text "Debt?" ]
                ]
            ]
        , tbody [] (List.map renderAccount accounts)
        ]


renderAccount : Account -> Html Msg
renderAccount account =
    tr []
        [ td [ onClick (OpenAccountEditor account) ] [ text account.name ]
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
