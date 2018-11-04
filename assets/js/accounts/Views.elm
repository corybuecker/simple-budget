module Accounts.Views exposing (editView, renderAccount, renderAccounts)

import Accounts.Messages exposing (..)
import Accounts.Models exposing (Account)
import Adjustments.Models exposing (Adjustment)
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import List exposing (map)
import List.Extra exposing (greedyGroupsOf)
import Model exposing (Msg(..))


editView : Account -> Html Accounts.Messages.Msg
editView model =
    div []
        [ input [ type_ "text", value model.name, onInput Accounts.Messages.NameUpdated ] []
        , input [ type_ "checkbox", checked model.debt, onClick Accounts.Messages.ToggleDebt ] []
        , input [ type_ "text", value (String.fromFloat model.balance), onInput Accounts.Messages.BalanceUpdated ] []
        , button [ onClick Accounts.Messages.SaveAccount, class "button" ] [ text "Save" ]
        , button [ onClick Accounts.Messages.DeleteAccount ] [ text "Delete" ]
        ]


renderAccounts : List Account -> Html Msg
renderAccounts accounts =
    div []
        [ button [ class "btn btn-primary", onClick CreateAccount ] [ text "New Account" ]
        , div [] (map renderAccountGroup (greedyGroupsOf 2 accounts))
        ]


renderAccountGroup : List Account -> Html Msg
renderAccountGroup group =
    div [ class "row" ] (map renderAccount group)


renderAccount : Account -> Html Msg
renderAccount account =
    div [ class "col-sm-6" ]
        [ div [ class "card" ]
            [ div [ class "card-body" ]
                (List.concat
                    [ [ h5 [ class "card-title", onClick (OpenAccountEditor account) ] [ text account.name ]
                      , div [] [ text (String.fromFloat account.balance) ]
                      , div []
                            [ text
                                (if account.debt then
                                    "True"

                                 else
                                    "False"
                                )
                            ]
                      , div [ onClick (CreateAdjustment account), class "button" ] [ text "Adjustment" ]
                      ]
                    , List.map renderAdjustment account.adjustments
                    ]
                )
            ]
        ]


renderAdjustment : Adjustment -> Html Msg
renderAdjustment adjustment =
    div []
        [ div [ onClick (OpenAdjustmentEditor adjustment), class "button" ] [ text adjustment.title ]
        , div [] [ text (String.fromFloat adjustment.total) ]
        ]
